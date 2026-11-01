//! Tests for Select component using `egui_kittest`

use armas_basic::components::SelectOption;
use armas_basic::prelude::*;
use egui_kittest::Harness;

/// Test that Select renders without panicking
#[test]
fn test_select_renders() {
    let options = vec![
        SelectOption::new("opt1", "Option 1"),
        SelectOption::new("opt2", "Option 2"),
        SelectOption::new("opt3", "Option 3"),
    ];

    let mut select = Select::new(options);

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select with pre-selected value
#[test]
fn test_select_with_selection() {
    let options = vec![
        SelectOption::new("apple", "Apple"),
        SelectOption::new("banana", "Banana"),
        SelectOption::new("cherry", "Cherry"),
    ];

    let mut select = Select::new(options).selected("banana");

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select with label
#[test]
fn test_select_with_label() {
    let options = vec![
        SelectOption::new("sm", "Small"),
        SelectOption::new("md", "Medium"),
        SelectOption::new("lg", "Large"),
    ];

    let mut select = Select::new(options).label("Size");

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select with custom placeholder
#[test]
fn test_select_with_placeholder() {
    let options = vec![
        SelectOption::new("red", "Red"),
        SelectOption::new("green", "Green"),
        SelectOption::new("blue", "Blue"),
    ];

    let mut select = Select::new(options).placeholder("Choose a color...");

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select with icons
#[test]
fn test_select_with_icons() {
    let options = vec![
        SelectOption::new("home", "Home").icon("🏠"),
        SelectOption::new("work", "Work").icon("💼"),
        SelectOption::new("school", "School").icon("🎓"),
    ];

    let mut select = Select::new(options);

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select with descriptions
#[test]
fn test_select_with_descriptions() {
    let options = vec![
        SelectOption::new("free", "Free Plan").description("Basic features, limited usage"),
        SelectOption::new("pro", "Pro Plan").description("All features, unlimited usage"),
        SelectOption::new("enterprise", "Enterprise").description("Custom solutions for teams"),
    ];

    let mut select = Select::new(options).label("Select Plan");

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select with disabled options
#[test]
fn test_select_with_disabled_options() {
    let options = vec![
        SelectOption::new("available", "Available"),
        SelectOption::new("unavailable", "Unavailable").disabled(true),
        SelectOption::new("coming_soon", "Coming Soon").disabled(true),
    ];

    let mut select = Select::new(options);

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select with search disabled
#[test]
fn test_select_not_searchable() {
    let options = vec![
        SelectOption::new("a", "Alpha"),
        SelectOption::new("b", "Beta"),
        SelectOption::new("c", "Gamma"),
    ];

    let mut select = Select::new(options).searchable(false);

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select with custom width
#[test]
fn test_select_custom_width() {
    let options = vec![
        SelectOption::new("1", "Option 1"),
        SelectOption::new("2", "Option 2"),
    ];

    let mut select = Select::new(options).width(300.0);

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test Select using builder API
#[test]
fn test_select_builder_api() {
    let mut select = Select::build(|s| {
        let _ = s.option("apple", "Apple").icon("🍎");
        let _ = s.option("banana", "Banana").icon("🍌");
        let _ = s
            .option("cherry", "Cherry")
            .icon("🍒")
            .description("Sweet red fruit");
    })
    .label("Fruit")
    .selected("apple");

    let mut harness = Harness::new_ui(|ui| {
        select.show(ui);
    });

    harness.run();
}

/// Test multiple selects
#[test]
fn test_multiple_selects() {
    let country_options = vec![
        SelectOption::new("us", "United States"),
        SelectOption::new("uk", "United Kingdom"),
        SelectOption::new("ca", "Canada"),
    ];

    let language_options = vec![
        SelectOption::new("en", "English"),
        SelectOption::new("es", "Spanish"),
        SelectOption::new("fr", "French"),
    ];

    let mut country_select = Select::new(country_options).label("Country");
    let mut language_select = Select::new(language_options).label("Language");

    let mut harness = Harness::new_ui(|ui| {
        ui.vertical(|ui| {
            country_select.show(ui);
            language_select.show(ui);
        });
    });

    harness.run();
}

/// Auto width is capped at 280pt even when the available width is larger
/// (911ca4e behaviour — pin it so a refactor can't silently drop the cap)
#[test]
fn test_select_auto_width_capped_at_280() {
    let options = vec![SelectOption::new("1", "Option 1")];

    let mut select = Select::new(options);
    let mut trigger_width = 0.0;

    {
        let mut harness = Harness::new_ui(|ui| {
            let response = select.show(ui);
            trigger_width = response.response.rect.width();
        });
        harness.set_size(egui::vec2(600.0, 400.0));
        harness.run();
    }

    assert!(
        (trigger_width - 280.0).abs() < 0.5,
        "expected ~280, got {trigger_width}"
    );
}

/// Width is floored at 16pt when the available width is tiny, so the
/// dropdown's `width - 16` subtractions can never go negative
#[test]
fn test_select_width_floored_when_avail_tiny() {
    let options = vec![SelectOption::new("1", "Option 1")];

    let mut select = Select::new(options);
    let mut trigger_width = 0.0;

    {
        let mut harness = Harness::new_ui(|ui| {
            // 5pt of horizontal room: below every subtraction in the dropdown
            ui.allocate_ui(egui::vec2(5.0, 100.0), |ui| {
                let response = select.show(ui);
                trigger_width = response.response.rect.width();
            });
        });
        harness.set_size(egui::vec2(600.0, 400.0));
        harness.run();
    }

    assert!(
        (trigger_width - 16.0).abs() < 0.5,
        "expected ~16, got {trigger_width}"
    );
}

/// Opening the dropdown inside a tiny container runs the `width - 8` /
/// `width - 16` subtractions at the 16pt floor — the paths that used to go
/// negative and trip egui's width asserts. The trigger is clicked via raw
/// pointer events because `is_open` has no public setter.
#[test]
fn test_select_dropdown_opens_in_tiny_container() {
    let options = vec![SelectOption::new("1", "Option 1")];

    let mut select = Select::new(options);

    {
        let mut harness = Harness::new_ui_state(
            |ui, last| {
                ui.allocate_ui(egui::vec2(5.0, 100.0), |ui| {
                    let response = select.show(ui);
                    *last = Some((response.response.rect, response.is_open));
                });
            },
            None, // (trigger rect, is_open)
        );
        harness.set_size(egui::vec2(600.0, 400.0));
        harness.run();

        let (rect, _) = harness.state().expect("first frame ran");
        let center = rect.center();
        harness.hover_at(center);
        harness.event(egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        });
        harness.event(egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        });
        harness.run_steps(2);

        let (_, is_open) = harness.state().expect("post-click frame ran");
        assert!(
            is_open,
            "dropdown should be open after clicking the trigger"
        );
    }
}
