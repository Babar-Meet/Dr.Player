//! The audio the seek bar draws, decoded from the file itself.
//!
//! Everything in here runs on one detached thread and talks to nothing: it reads a path, decodes
//! the file's own first audio track, and hands the caller a sequence of `WaveMsg`. It has no idea a
//! window, a webview or a seek bar exists, because the one thing this file must not do is hold a
//! borrow that reaches back into the app's event loop. `std::thread::spawn` would not allow it, and
//! that is the guarantee the close-within-the-first-second case rests on.
//!
//! Nothing in here can unwind into a process panic and nothing can hang: every fallible call is
//! matched, the one place a third-party decoder could panic is wrapped so that a panic becomes
//! `Failed` rather than an abort, no loop waits on anything but the file, and no loop runs longer
//! than decoding `WAVE_MAX_SECONDS` of audio takes.
//!
//! The error policy has one answer. `Failed` means "draw the plain line instead", which is what the
//! bar already does and what a file with no audio, a container this build cannot demux, and a
//! truncated download all have in common. There is no third outcome and no partial answer: a
//! waveform that stops short of the end of the track is worse than no waveform, because its time
//! axis is the video's and it would be claiming audio where there is none.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{Track, TrackType};
use symphonia::core::io::MediaSourceStream;

/// One column of the waveform: the largest absolute sample seen in a `WAVE_BUCKET_SECONDS`
/// window of the audio track, as a fraction of full scale.
pub type WavePeak = f32;

/// Length of one bucket, in seconds. Frozen: 20 ms.
///
/// About one frame at 50 fps and two at 25, so a bucket never smears across more than about two
/// frames of picture, and fine enough that a drum hit is a column of its own rather than a smear.
/// It is a constant rather than one bucket per device pixel because the bar changes width on every
/// resize: a pixel grid would mean re-decoding on every drag of the window edge, while a fixed grid
/// is decoded once and re-aggregated from memory forever. Both ends of the bar's real width range
/// survive it. At the narrowest layout, about 162 CSS px and so 324 device columns at DPR 2, a
/// ten-minute file's 30,000 buckets are an over-sample that max-aggregation folds down; at the
/// widest, 1300 CSS px and 2600 columns, the same file is an under-sample that max-aggregation
/// still catches every transient in.
pub const WAVE_BUCKET_SECONDS: f64 = 0.02;

/// Peaks per message across the bridge. Frozen: 512.
///
/// A batch is about 3.1 KB of JSON and one event-loop wake. A ten-minute file is 59 of them, spread
/// over a decode that takes seconds, on a loop that spends its life in `ControlFlow::Wait`.
pub const WAVE_BATCH_PEAKS: usize = 512;

/// How much audio one decode will read before it stops and emits what it has. Chosen, not measured.
///
/// Two things grow with the length of a track and neither is bounded by anything the file will do
/// on its own: the decode holds one core for as long as the audio lasts, and the page's peak array
/// grows at 50 entries a second of audio and is held for the life of the window. Two hours is
/// 360,000 entries, about 2.9 MB of `f64`, read once per paint at 4 Hz, and a decode that stops
/// there has cost seconds rather than hours. The longest committed fixture is 653.792 s
/// (`elephants_dream_1024x576_30s.mp4`) and the owner's own large file is minutes, so nothing
/// ordinary comes anywhere near it.
///
/// Reaching it is finishing, not failing, and it deliberately does not take the fallback. The last
/// bucket is closed, what arrived is flushed, and the terminal message is `Done`, which is the same
/// answer a track that ended on its own gives: the page draws what it has and lays the minimum line
/// over the rest, which is what it already does for a file whose audio is shorter than its video.
pub const WAVE_MAX_SECONDS: f64 = 7200.0;

/// What one message across the bridge carries. `Failed` is the fallback signal; see the note above.
pub enum WaveMsg {
    /// A run of consecutive buckets. `start` indexes `peaks[0]` within the whole track.
    Chunk {
        /// Index of `peaks[0]` in the whole track, counted in buckets from t=0.
        start: usize,
        /// 1..=`WAVE_BATCH_PEAKS` values, each linear amplitude in 0.0..=1.0.
        peaks: Vec<WavePeak>,
    },
    /// The two numbers the page must not hardcode. It goes out once before the first chunk and
    /// again when the decode ends.
    Meta {
        /// `WAVE_BUCKET_SECONDS`, stated rather than assumed.
        bucket_seconds: f64,
        /// Seconds of *audio*, from decoded frames over the sample rate. The first one is the
        /// container's own figure where it states one and 0.0 where it does not, which the page
        /// reads as "not known yet"; the second is the decoded truth and replaces it.
        duration_seconds: f64,
    },
    /// The decode reached the end of the track on its own.
    Done,
    /// Nothing to draw. The page keeps or restores the plain line.
    Failed,
}

/// Decode the file at `path` and hand every batch to `emit` as it is produced.
///
/// Returns the terminal message, which the caller emits. This function emits `Meta` and `Chunk`
/// and nothing else, so that "the decode failed" has exactly one shape whichever of the failure
/// paths produced it. Returning rather than emitting is also what keeps an early `return` in the
/// middle from silently producing no message at all, which is the one outcome the page cannot tell
/// apart from a decode that is still working.
///
/// The caller does not join the thread this runs on. Once the event loop is gone every send out of
/// it fails and is discarded, and the file handle inside it is reclaimed with the process.
pub fn extract(path: PathBuf, emit: impl Fn(WaveMsg)) -> WaveMsg {
    // Symphonia is third-party code over untrusted bytes, and a panic in it on a malformed file
    // would print through the hook in main and take the process down at close. There is no way to
    // stop a decoder panicking from inside its own loop, so the whole extraction is caught and a
    // panic becomes Failed: the plain line, and a line on stderr, which is this program's stated
    // diagnostic channel (README:49). A library bug is worth saying out loud; a crash is not.
    match catch_unwind(AssertUnwindSafe(|| decode(&path, &emit))) {
        Ok(terminal) => terminal,
        Err(_) => WaveMsg::Failed,
    }
}

fn decode(path: &Path, emit: &impl Fn(WaveMsg)) -> WaveMsg {
    // The file handle lives here, in this thread, and goes when the thread does.
    // MediaSourceStream reads through the Box rather than slurping it, so a 133 MB file never sits
    // in memory and the decode's cost is the decode and not the read.
    let Ok(file) = std::fs::File::open(path) else {
        return WaveMsg::Failed;
    };
    let stream = MediaSourceStream::new(Box::new(file), Default::default());

    // The extension is only a hint: the probe still has to recognise the container from the bytes.
    // It is worth giving because an mp4 and a mov are one demuxer under two file extensions, and
    // without it the first kilobyte is read twice to work that out.
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(extension);
    }

    let probe = symphonia::default::get_probe();
    let Ok(mut format) = probe.probe(&hint, stream, Default::default(), Default::default()) else {
        return WaveMsg::Failed;
    };

    // Four of the committed fixtures have no audio stream at all, and a container none of the
    // compiled demuxers recognises (CAF, AIFF, a file cut short) fails above instead. Both answers
    // are the same one: no track, no waveform.
    let Some(track) = format.default_track(TrackType::Audio) else {
        return WaveMsg::Failed;
    };
    let Some(params) = track.codec_params.as_ref().and_then(|p| p.audio()) else {
        return WaveMsg::Failed;
    };
    let track_id = track.id;
    let declared = declared_seconds(track, params.sample_rate);

    let codecs = symphonia::default::get_codecs();
    let Ok(mut decoder) = codecs.make_audio_decoder(params, &Default::default()) else {
        return WaveMsg::Failed;
    };

    let mut bins = Bins::new(declared, emit);
    // One buffer reused for every packet rather than one per packet: a stereo AAC frame is 1024
    // frames, so a ten-minute file is about 27,000 packets, and 27,000 allocations is a cost the
    // decoder never asked anybody to pay.
    let mut interleaved: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            // End of the track, reached by the file itself and not by an error.
            Ok(None) => break,
            // The one documented source of this is a chained physical stream inside an Ogg file,
            // which asks the consumer to rebuild its decoders and start over. Nothing this build
            // decodes produces one, and answering it by restarting risks an unbounded loop over a
            // file that keeps asking. The plain line is the honest answer.
            Err(Error::ResetRequired) => return WaveMsg::Failed,
            // Anything else from the demuxer is fatal. The track list cannot be walked past it,
            // and a waveform that stopped where the demuxer stopped would be measured against the
            // video's clock, so it would be claiming audio where there is none.
            Err(_) => return WaveMsg::Failed,
        };

        // Metadata is never read here, so the queue is drained rather than left to grow for the
        // length of the decode; a file with chapters appends one revision per chapter.
        while !format.metadata().is_latest() {
            format.metadata().pop();
        }

        // A multi-track container hands back the video's packets too, and those carry no audio.
        if packet.track_id != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            // One undecodable packet is normal in a real file and costs one 20 ms column, which
            // the page draws at its minimum height: a visible gap is the truth about it.
            Err(Error::DecodeError(_)) | Err(Error::IoError(_)) => continue,
            // The decoder is saying its own state no longer applies to this packet. Reset and
            // carry on from here, which is what the codec trait documents for this case.
            Err(Error::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(_) => return WaveMsg::Failed,
        };

        if decoded.is_empty() {
            continue;
        }
        fold(&mut bins, &decoded, &mut interleaved);
        // The budget, checked once per packet and not per sample: a packet is roughly 20 ms of
        // audio, so this is one comparison per column of the bar. Falling out of the loop is the
        // track ending, which is why it goes through the same finish() a file that ran out does
        // rather than through an error.
        if bins.seconds() >= WAVE_MAX_SECONDS {
            break;
        }
    }

    // The decoder's tail is not a separate read. In this version of symphonia the AAC, MP3 and
    // Vorbis decoders all return the last packet's samples from last_decoded and none of them adds
    // anything to it on finalize, so the last block the loop folded in above IS the tail. Reading
    // it again would count the final packet's samples twice and invent a run of buckets past the
    // end of the audio.
    bins.finish()
}

/// Copy one decoded buffer out as interleaved f32 and fold it into the buckets.
///
/// The copy is what makes the peak a plain scan: symphonia hands back a buffer in whatever sample
/// format the codec produces, and f32 at full linear scale is the one format in which a sample's
/// magnitude already means "fraction of full scale". No dB anywhere, because a display scale would
/// be a second convention somebody would have to convert correctly later.
fn fold(
    bins: &mut Bins<'_>,
    decoded: &symphonia::core::audio::GenericAudioBufferRef<'_>,
    interleaved: &mut Vec<f32>,
) {
    let planes = decoded.num_planes();
    let frames = decoded.frames();
    interleaved.resize(planes * frames, 0.0);
    decoded.copy_to_slice_interleaved(interleaved.as_mut_slice());
    bins.push(interleaved, planes, frames, decoded.spec().rate());
}

/// The audio length the container states, where it states one at all.
///
/// This is the provisional figure only. `Meta` carries it on the way in so the page can place the
/// audio-less tail from the first batch, and the decoded figure replaces it when the track ends.
/// 0.0 means "not stated", which the page reads the same as "not known yet".
fn declared_seconds(track: &Track, sample_rate: Option<u32>) -> f64 {
    match (track.num_frames, sample_rate) {
        (Some(frames), Some(rate)) if frames > 0 && rate > 0 => frames as f64 / rate as f64,
        _ => 0.0,
    }
}

/// 1 / `WAVE_BUCKET_SECONDS` as an integer, which is what keeps the bucket boundary exact.
const WAVE_DENOM: u64 = 50;

/// The bucket accumulator, and nothing else.
///
/// Private to `decode`, and holding only what a bucket boundary needs: the open bucket's running
/// maximum, where the next one closes, and the batch being filled. Why the bucket is 20 ms wide
/// and why the aggregate is a maximum rather than a mean are said at `WAVE_BUCKET_SECONDS`; this
/// is the arithmetic those sentences need in order to be true.
struct Bins<'a> {
    emit: &'a dyn Fn(WaveMsg),
    /// Where the next bucket closes, held as `boundary_k = k * rate / WAVE_DENOM` and compared
    /// against `frame * WAVE_DENOM`. Integer rather than floating point because a 20 ms bucket is
    /// 882 frames at 44.1 kHz, 960 at 48 kHz and 220.5 at 11.025 kHz: a float accumulator drifts
    /// over the ten minutes this runs for, and `frames * 50 >= next` costs a multiply instead of a
    /// division in the one loop that runs tens of millions of times.
    next_boundary: u64,
    /// The rate the boundary arithmetic is currently in, and the last one seen. Zero until the
    /// first packet, so a decoder that changes rate mid-track restarts the clock instead of mixing
    /// two clocks into one counter.
    rate: u32,
    /// The largest absolute sample in the bucket currently open.
    open: WavePeak,
    /// Frames decoded in total. The audio length is this over `rate`, and it is the only clock in
    /// this path: the decoder's own frame counter, never the container's timestamps and never the
    /// video's, so the bar's two halves cannot disagree about what second they are on.
    total_frames: u64,
    /// The batch being filled, and where it starts within the whole track.
    batch: Vec<WavePeak>,
    batch_start: usize,
    /// Whether `Meta` has gone out. It goes with the first bucket rather than at construction, so
    /// it is never sent for a file that produced no audio at all.
    announced: bool,
    /// The container's own figure, held until the first `Meta` is built.
    declared: f64,
}

impl<'a> Bins<'a> {
    fn new(declared: f64, emit: &'a impl Fn(WaveMsg)) -> Self {
        Bins {
            emit,
            next_boundary: 0,
            rate: 0,
            open: 0.0,
            total_frames: 0,
            batch: Vec::with_capacity(WAVE_BATCH_PEAKS),
            batch_start: 0,
            announced: false,
            declared,
        }
    }

    /// Fold one decoded packet in. `samples` holds `planes` values for each of `frames` frames.
    ///
    /// The peak is the maximum of the absolute value, over every channel and over every frame in
    /// the bucket, never an average of the two: averaging channels drops a hard-panned one into
    /// the middle, and averaging a bucket drops the transient that is the reason the bucket is
    /// 20 ms wide. One comparison per sample, against a multiply-add and a square root for an RMS.
    fn push(&mut self, samples: &[f32], planes: usize, frames: usize, rate: u32) {
        // A rate of zero cannot bound a bucket, and no decoder produces one. Skipping the packet
        // is the only answer that cannot divide by it.
        if rate == 0 || planes == 0 || frames == 0 {
            return;
        }
        if self.rate != rate {
            // Close what is open under the old rate before the clock moves, so two rates never
            // share a bucket.
            if self.rate != 0 {
                self.close();
            }
            self.rate = rate;
            self.next_boundary = rate as u64;
        }
        let rate64 = rate as u64;

        for frame in 0..frames {
            // The boundary is tested before this frame goes in, so a bucket is exactly the frames
            // [k * rate/50, (k+1) * rate/50) and nothing spills across the line.
            while self.total_frames * WAVE_DENOM >= self.next_boundary {
                self.close();
                self.next_boundary += rate64;
            }
            let base = frame * planes;
            // base + planes is bounded by samples.len() by construction: that is how many values
            // fold just wrote for exactly these frames and planes.
            let peak = samples[base..base + planes]
                .iter()
                .fold(0.0f32, |acc, s| acc.max(s.abs()));
            if peak > self.open {
                self.open = peak;
            }
            self.total_frames += 1;
        }
    }

    /// Seconds of audio folded in so far, from the decoder's own frame counter. That is the only
    /// clock this file keeps, and the only one `WAVE_MAX_SECONDS` is measured against.
    fn seconds(&self) -> f64 {
        if self.rate == 0 {
            return 0.0;
        }
        self.total_frames as f64 / self.rate as f64
    }

    /// Close the bucket that is open and start the next one.
    fn close(&mut self) {
        if !self.announced {
            self.announced = true;
            (self.emit)(WaveMsg::Meta {
                bucket_seconds: WAVE_BUCKET_SECONDS,
                duration_seconds: self.declared,
            });
        }
        // A NaN out of a decoder would poison every comparison it took part in, so a bucket's
        // value is clamped on the way out rather than trusted. Full scale is 1.0 and that is the
        // whole declared range: this is linear amplitude, not dB.
        let peak = if self.open.is_finite() {
            self.open.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.open = 0.0;
        self.batch.push(peak);
        if self.batch.len() >= WAVE_BATCH_PEAKS {
            self.flush();
        }
    }

    /// Send the batch being filled, if there is one, and start the next.
    fn flush(&mut self) {
        if self.batch.is_empty() {
            return;
        }
        let start = self.batch_start;
        self.batch_start += self.batch.len();
        let peaks = std::mem::replace(&mut self.batch, Vec::with_capacity(WAVE_BATCH_PEAKS));
        (self.emit)(WaveMsg::Chunk { start, peaks });
    }

    /// Close the last bucket, send what is left, and answer with the terminal message.
    ///
    /// The track's final packet ends mid-bucket and that bucket is a normal one: closed with
    /// whatever arrived, emitted like any other. No padding, no zero-fill, no dropping. The
    /// decoded length is `total_frames / rate`, the audio's true length and generally not a
    /// multiple of 20 ms, so the time-to-bucket map handles the short final span with no case of
    /// its own.
    fn finish(&mut self) -> WaveMsg {
        // A track that decoded to nothing at all is not a waveform, it is a decoder that did not
        // work, and an empty canvas is the one thing the page cannot tell from a decode still
        // running. The fallback is the honest answer.
        if self.total_frames == 0 || self.rate == 0 {
            return WaveMsg::Failed;
        }
        self.close();
        self.flush();
        let duration_seconds = self.total_frames as f64 / self.rate as f64;
        (self.emit)(WaveMsg::Meta {
            bucket_seconds: WAVE_BUCKET_SECONDS,
            duration_seconds,
        });
        WaveMsg::Done
    }
}
