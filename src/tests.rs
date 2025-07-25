use crate::events::{Event, PropertyData};
use crate::*;

use std::thread;
use std::time::Duration;

#[test]
fn initializer() {
    let mpv = Mpv::with_initializer(|init| {
        init.set_property("osc", true)?;
        init.set_property("input-default-bindings", true)?;
        init.set_property("volume", 30)?;

        Ok(())
    })
    .unwrap();

    assert_eq!(true, mpv.get_property("osc").unwrap());
    assert_eq!(true, mpv.get_property("input-default-bindings").unwrap());
    assert_eq!(30i64, mpv.get_property("volume").unwrap());
}

#[test]
fn properties() {
    let mpv = Mpv::new().unwrap();
    mpv.set_property("volume", 0).unwrap();
    mpv.set_property("vo", "null").unwrap();
    mpv.set_property("ytdl-format", "best[width<240]").unwrap();
    mpv.set_property("sub-gauss", 0.6).unwrap();

    assert_eq!(0i64, mpv.get_property("volume").unwrap());
    let vo: MpvStr = mpv.get_property("vo").unwrap();
    assert_eq!("null", &*vo);
    assert_eq!(true, mpv.get_property("ytdl").unwrap());
    let subg: f64 = mpv.get_property("sub-gauss").unwrap();
    assert_eq!(
        0.6,
        f64::round(subg * f64::powi(10.0, 4)) / f64::powi(10.0, 4)
    );
    mpv.command(
        "loadfile",
        &["test-data/speech_12kbps_mb.wav", "append-play"],
    )
    .unwrap();
    thread::sleep(Duration::from_millis(250));

    let title: MpvStr = mpv.get_property("media-title").unwrap();
    assert_eq!(&*title, "speech_12kbps_mb.wav");
}

macro_rules! assert_event_occurs {
    ($ctx:ident, $timeout:literal, $( $expected:pat),+) => {
        loop {
            match $ctx.wait_event($timeout) {
                $( Some($expected) )|+ => {
                    break;
                },
                None => {
                    continue
                },
                other => panic!("Event did not occur, got: {:?}", other),
            }
        }
    }
}

#[test]
fn events() {
    let mut mpv = Mpv::new().unwrap();
    mpv.disable_deprecated_events().unwrap();

    mpv.observe_property("volume", Format::Int64, 0).unwrap();
    mpv.observe_property("media-title", Format::String, 1)
        .unwrap();

    mpv.set_property("vo", "null").unwrap();

    // speed up playback so test finishes faster
    mpv.set_property("speed", 100).unwrap();

    assert_event_occurs!(
        mpv,
        3.,
        Ok(Event::PropertyChange {
            name: "volume",
            change: PropertyData::Int64(100),
            reply_userdata: 0,
        })
    );

    mpv.set_property("volume", 0).unwrap();
    assert_event_occurs!(
        mpv,
        10.,
        Ok(Event::PropertyChange {
            name: "volume",
            change: PropertyData::Int64(0),
            reply_userdata: 0,
        })
    );
    assert!(mpv.wait_event(3.).is_none());
    mpv.command("loadfile", &["test-data/jellyfish.mp4", "append-play"])
        .unwrap();
    assert_event_occurs!(mpv, 10., Ok(Event::StartFile));
    assert_event_occurs!(
        mpv,
        10.,
        Ok(Event::PropertyChange {
            name: "media-title",
            change: PropertyData::Str("jellyfish.mp4"),
            reply_userdata: 1,
        })
    );
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::FileLoaded));
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::VideoReconfig));

    mpv.command("loadfile", &["test-data/speech_12kbps_mb.wav", "replace"])
        .unwrap();
    assert_event_occurs!(mpv, 3., Ok(Event::VideoReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::VideoReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::EndFile(mpv_end_file_reason::Stop)));
    assert_event_occurs!(mpv, 3., Ok(Event::StartFile));
    assert_event_occurs!(
        mpv,
        3.,
        Ok(Event::PropertyChange {
            name: "media-title",
            change: PropertyData::Str("speech_12kbps_mb.wav"),
            reply_userdata: 1,
        })
    );
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::VideoReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::FileLoaded));
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 3., Ok(Event::PlaybackRestart));
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert_event_occurs!(mpv, 10., Ok(Event::EndFile(mpv_end_file_reason::Eof)));
    assert_event_occurs!(mpv, 3., Ok(Event::AudioReconfig));
    assert!(mpv.wait_event(3.).is_none());
}

#[test]
fn config_file() {
    let mpv = Mpv::with_initializer(|init| {
        init.load_config("test-data/mpv.conf").unwrap();

        Ok(())
    })
    .unwrap();
    assert_eq!(mpv.get_property::<i64>("volume").unwrap(), 50);
}
