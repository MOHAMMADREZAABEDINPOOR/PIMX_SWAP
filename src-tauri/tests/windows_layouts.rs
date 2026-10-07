//! Uses OS-supplied layouts, never static keyboard tables. Serial execution recommended.
use pimxswap_lib::{config::Config, detection, layout::Engine};
use windows::{core::PCWSTR, Win32::UI::Input::KeyboardAndMouse::*};
fn load(id: &str) -> String {
    let id: Vec<u16> = id.encode_utf16().chain(Some(0)).collect();
    // SAFETY: a valid Windows KLID, no KLF_ACTIVATE; loads only into this test process.
    let h = unsafe { LoadKeyboardLayoutW(PCWSTR(id.as_ptr()), KLF_NOTELLSHELL) }
        .expect("Windows layout load");
    assert!(!h.0.is_null(), "Windows layout is available");
    format!("{:016X}", h.0 as usize)
}
#[test]
fn real_windows_language_pairs_and_sentences() {
    let en = load("00000409");
    let fa = load("00000429");
    let ar = load("00000401");
    let ru = load("00000419");
    let de = load("00000407");
    let mut engine = Engine::default();
    assert_eq!(engine.convert("sghl", &en, &fa).unwrap().text, "سلام");
    assert_eq!(engine.convert("سلام", &fa, &en).unwrap().text, "sghl");
    assert_eq!(
        engine.convert("ghbdtn vbh", &en, &ru).unwrap().text,
        "привет мир"
    );
    assert_eq!(
        engine.convert("привет мир", &ru, &en).unwrap().text,
        "ghbdtn vbh"
    );
    assert_eq!(
        engine.convert("hallo welt yz YZ", &en, &de).unwrap().text,
        "hallo welt zy ZY"
    );
    for (layout, sentences) in [
        (&fa, vec!["سلام دنیا", "صبح بخیر", "من خوب هستم"]),
        (&ar, vec!["مرحبا العالم", "صباح الخير", "كيف حالك"]),
        (&ru, vec!["привет мир", "доброе утро", "как дела"]),
        (&de, vec!["hallo welt", "guten morgen", "wie geht es dir"]),
    ] {
        for sentence in sentences {
            let wrong = engine.convert(sentence, layout, &en).unwrap();
            assert_eq!(wrong.unmapped, 0, "{sentence}");
            let back = engine.convert(&wrong.text, &en, layout).unwrap();
            assert_eq!(back.text, sentence, "{sentence} via {}", wrong.text);
        }
    }
    assert_eq!(
        engine.convert("123! @# \n🙂", &en, &en).unwrap().text,
        "123! @# \n🙂"
    );
    assert_eq!(
        engine.convert("HELLO world!", &en, &en).unwrap().text,
        "HELLO world!"
    );
    let mixed = engine.convert("sghl 🙂 فارسی", &en, &fa).unwrap();
    assert!(mixed.text.starts_with("سلام 🙂"));
    assert!(mixed.text.ends_with("فارسی"));
}
#[test]
fn auto_fix_correct_text_mixed_text_and_ambiguity() {
    let en = load("00000409");
    let fa = load("00000429");
    let _ar = load("00000401");
    let _ru = load("00000419");
    let _de = load("00000407");
    let layouts = pimxswap_lib::layout::installed();
    let mut engine = Engine::default();
    let config = Config::default();
    let result = detection::analyze(&mut engine, "sghl", &layouts, &config, &en).unwrap();
    assert_eq!(result.action, "apply");
    assert_eq!(result.candidates[0].text, "سلام");
    assert_eq!(result.candidates[0].target, fa);
    for text in [
        "hello world",
        "سلام my friend 123!",
        "hallo welt",
        "привет мир",
    ] {
        assert_eq!(
            detection::analyze(&mut engine, text, &layouts, &config, &en)
                .unwrap()
                .action,
            "unchanged",
            "{text}"
        );
    }
    assert_ne!(
        detection::analyze(&mut engine, "xy", &layouts, &config, &en)
            .unwrap()
            .action,
        "apply"
    );
}
#[test]
fn german_altgr_dead_keys_and_composed_unicode() {
    let en = load("00000409");
    let de = load("00000407");
    let mut e = Engine::default();
    assert_eq!(e.convert("€", &de, &de).unwrap().text, "€");
    assert_eq!(e.convert("é â", &de, &de).unwrap().text, "é â");
    let keys = e.convert("é â", &de, &en).unwrap().text;
    assert_eq!(e.convert(&keys, &en, &de).unwrap().text, "é â");
    assert_eq!(e.convert("zZ", &en, &de).unwrap().text, "yY");
}
#[test]
fn catalog_layouts_load_on_demand_and_keep_stable_ids() {
    use pimxswap_lib::layout;
    let catalog = layout::available();
    assert!(catalog.len() > 100, "Full Windows keyboard layout catalog");
    let french = catalog
        .iter()
        .find(|l| l.id.eq_ignore_ascii_case("0000040c"))
        .unwrap();
    let english = catalog.iter().find(|l| l.id == "00000409").unwrap();
    let persian = catalog.iter().find(|l| l.id == "00000429").unwrap();
    let mut engine = Engine::default();
    assert_eq!(
        engine
            .convert("qwerty", &english.id, &french.id)
            .unwrap()
            .text,
        "azerty"
    );
    assert_eq!(
        engine
            .convert("اثممخ", &persian.id, &english.id)
            .unwrap()
            .text,
        "hello"
    );
    engine.clear();
    assert_eq!(
        engine
            .convert("azerty", &french.id, &english.id)
            .unwrap()
            .text,
        "qwerty"
    );
    assert!(layout::available().iter().any(|l| l.id == french.id));
    let config = Config {
        source: persian.id.clone(),
        target: english.id.clone(),
        ..Config::default()
    };
    let analysis = detection::analyze(
        &mut engine,
        "اثممخ",
        &layout::detection_layouts(&config),
        &config,
        &persian.id,
    )
    .unwrap();
    assert_eq!(analysis.action, "apply");
    assert_eq!(analysis.candidates[0].text, "hello");
}
#[test]
fn configured_pair_converts_unfinished_sentences_without_dictionary_gate() {
    use pimxswap_lib::layout;
    let en = load("00000409");
    let fa = load("00000429");
    let config = Config {
        source: en.clone(),
        target: fa.clone(),
        ..Config::default()
    };
    let mut engine = Engine::default();
    for text in ["sghl lk o,", "sghl lk o", "sghl a", "lk o,", "xy"] {
        let expected = engine.convert(text, &en, &fa).unwrap().text;
        let analysis = detection::analyze(
            &mut engine,
            text,
            &layout::detection_layouts(&config),
            &config,
            &en,
        )
        .unwrap();
        assert_eq!(analysis.action, "apply", "{text}");
        assert_eq!(
            analysis.candidates[0].text, expected,
            "Every character in the incomplete input is converted"
        );
    }
    assert_eq!(
        engine.convert("sghl lk o,", &en, &fa).unwrap().text,
        "سلام من خو"
    );
    let english = detection::analyze(
        &mut engine,
        "hello world",
        &layout::detection_layouts(&config),
        &config,
        &en,
    )
    .unwrap();
    assert_eq!(english.action, "unchanged");
    let hello = detection::analyze(
        &mut engine,
        "اثممخ",
        &layout::detection_layouts(&config),
        &config,
        &fa,
    )
    .unwrap();
    assert_eq!(hello.action, "apply");
    assert_eq!(hello.candidates[0].text, "hello");
}
