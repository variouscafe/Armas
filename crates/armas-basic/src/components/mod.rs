//! UI components
//!
//! All Armas components in a flat module structure.

pub mod accordion;
pub mod alert;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod button_group;
pub mod calendar;
pub mod card;
pub mod carousel;
pub mod checkbox;
pub mod command;
pub mod content;
pub mod context_menu;
pub mod date_picker;
pub mod dialog;
pub mod drawer;
pub mod dropdown_menu;
pub mod hover_card;
pub mod icon_button;
pub mod input;
pub mod input_group;
pub mod kbd;
pub mod loading;
pub mod menubar;
pub mod number_field;
pub mod pagination;
pub mod popover;
pub mod progress;
pub mod radio;
pub mod range_slider;
pub mod select;
pub mod separator;
pub mod sheet;
pub mod sidebar;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod textarea;
pub mod toast;
pub mod toggle;
pub mod tooltip;
pub mod tree_view;

// Re-exports
pub use accordion::{Accordion, AccordionResponse};
pub use alert::{alert, alert_destructive, Alert, AlertResponse, AlertVariant};
pub use avatar::{Avatar, AvatarShape, AvatarSize};
pub use badge::{Badge, BadgeResponse, BadgeVariant, NotificationBadge};
pub use breadcrumb::{Breadcrumb, BreadcrumbResponse};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use button_group::{ButtonGroup, ButtonGroupOrientation, ButtonGroupResponse};
pub use calendar::{Calendar, CalendarResponse};
pub use card::{Card, CardResponse, CardVariant};
pub use carousel::{Carousel, CarouselOrientation, CarouselResponse};
pub use checkbox::{Checkbox, CheckboxResponse};
pub use command::{Command, CommandResponse};
pub use content::ContentContext;
pub use context_menu::{ContextMenu, ContextMenuResponse};
pub use date_picker::{Date, DatePicker, DatePickerResponse};
pub use dialog::{dialog_footer, Dialog, DialogResponse, DialogSize};
pub use drawer::{Drawer, DrawerResponse, DrawerSnapPoint};
pub use dropdown_menu::{DropdownMenu, DropdownMenuResponse, MenuBuilder};
pub use hover_card::{HoverCard, HoverCardResponse};
pub use icon_button::IconButton;
pub use input::{Input, InputState, InputVariant, SearchInput};
pub use input_group::{InputGroup, InputGroupResponse};
pub use kbd::Kbd;
pub use loading::{Skeleton, Spinner};
pub use menubar::{Menubar, MenubarResponse};
pub use number_field::{NumberField, NumberFieldResponse};
pub use pagination::{Pagination, PaginationResponse};
pub use popover::{Popover, PopoverColor, PopoverPosition, PopoverResponse, PopoverStyle};
pub use progress::{CircularProgressBar, Progress};
pub use radio::{Radio, RadioGroup, RadioGroupResponse, RadioResponse, RadioSize};
pub use range_slider::{RangeSlider, RangeSliderResponse};
pub use select::{Select, SelectOption, SelectResponse};
pub use separator::{Separator, SeparatorOrientation};
pub use sheet::{Sheet, SheetResponse, SheetSide, SheetSize};
pub use sidebar::{CollapsibleMode, Sidebar, SidebarResponse, SidebarState, SidebarVariant};
pub use slider::{Slider, SliderResponse};
pub use switch::{Switch, SwitchResponse, SwitchSize};
pub use tabs::{Tabs, TabsResponse};
pub use textarea::Textarea;
pub use toast::{ToastId, ToastManager, ToastPosition, ToastVariant};
pub use toggle::{
    Toggle, ToggleGroup, ToggleGroupResponse, ToggleGroupSize, ToggleGroupType, ToggleGroupVariant,
    ToggleResponse, ToggleSize, ToggleVariant,
};
pub use tooltip::{tooltip, tooltip_with, Tooltip, TooltipPosition};
pub use tree_view::{TreeItem, TreeView, TreeViewResponse};

// Backwards compatibility aliases
#[doc(hidden)]
pub use tree_view::{Browser, BrowserItem, BrowserResponse};

// ============================================================================
// Shared width-knob policy
// ============================================================================

/// Width knobs accept finite positive values; anything else (NaN, ±∞, zero,
/// negative — e.g. derived from an unmeasured container) falls back to
/// `default`. "No cap" is a large *finite* value (`f32::MAX`): an infinite
/// knob poisons finite arithmetic at use sites (offscreen anchors, infinite
/// allocations).
pub(crate) const fn sanitize_width(raw: f32, default: f32) -> f32 {
    crate::sanitize::finite_above_or(raw, 0.0, default)
}

/// [`sanitize_width`] for explicit-width knobs, classifying a single raw
/// value: finite positive → set; garbage counts as unset.
pub(crate) const fn sanitize_opt_width(raw: f32) -> Option<f32> {
    let sane = sanitize_width(raw, 0.0);
    if sane > 0.0 {
        Some(sane)
    } else {
        None
    }
}

/// [`sanitize_width`]'s policy for height knobs, under its own name so
/// width and height audits stay distinct (a width-specific tightening
/// must not silently change heights, and vice versa).
pub(crate) const fn sanitize_height(raw: f32, default: f32) -> f32 {
    crate::sanitize::finite_above_or(raw, 0.0, default)
}

/// [`sanitize_height`] for explicit-height knobs, classifying a single
/// raw value: finite positive → set; garbage counts as unset.
pub(crate) const fn sanitize_opt_height(raw: f32) -> Option<f32> {
    let sane = sanitize_height(raw, 0.0);
    if sane > 0.0 {
        Some(sane)
    } else {
        None
    }
}

/// Positive-size knobs that are neither width nor height (font sizes,
/// icon sizes): finite positive passes, garbage falls back to `default`.
pub(crate) const fn sanitize_size(raw: f32, default: f32) -> f32 {
    crate::sanitize::finite_above_or(raw, 0.0, default)
}

/// [`sanitize_size`] for explicit-size knobs: garbage counts as unset.
pub(crate) const fn sanitize_opt_size(raw: f32) -> Option<f32> {
    let sane = sanitize_size(raw, 0.0);
    if sane > 0.0 {
        Some(sane)
    } else {
        None
    }
}

/// Padding and spacing knobs, where zero is legal (flush / no gap / no
/// delay — also sentinel-zero heights): a finite non-negative value
/// passes, garbage falls back to `default`.
pub(crate) const fn sanitize_padding(raw: f32, default: f32) -> f32 {
    crate::sanitize::finite_at_least_or(raw, 0.0, default)
}

/// [`sanitize_padding`] for explicit-padding knobs: garbage counts as
/// unset.
pub(crate) const fn sanitize_opt_padding(raw: f32) -> Option<f32> {
    let sane = crate::sanitize::finite_at_least_or(raw, 0.0, -1.0);
    if sane >= 0.0 {
        Some(sane)
    } else {
        None
    }
}

/// Corner-radius knobs: finite non-negative, capped where the u8
/// `CornerRadius` conversion saturates anyway.
pub(crate) const fn sanitize_radius(raw: f32, default: f32) -> f32 {
    crate::sanitize::finite_at_least_or(raw, 0.0, default).min(255.0)
}

/// [`sanitize_radius`] for explicit-radius knobs: garbage counts as
/// unset.
pub(crate) const fn sanitize_opt_radius(raw: f32) -> Option<f32> {
    let sane = crate::sanitize::finite_at_least_or(raw, 0.0, -1.0);
    if sane >= 0.0 {
        Some(sane.min(255.0))
    } else {
        None
    }
}

/// Value-domain knobs (slider bounds and defaults, number-field bounds):
/// any finite value passes; garbage counts as unset.
pub(crate) const fn sanitize_opt_value(raw: f32) -> Option<f32> {
    if raw.is_finite() {
        Some(raw)
    } else {
        None
    }
}

/// Domain-bound pairs (slider ranges): one-sided garbage heals around
/// the surviving bound (a 1.0 span), double garbage falls back to
/// 0..=1, and an inverted pair is swapped so `min <= max` always holds.
pub(crate) const fn sanitize_bounds(min: f32, max: f32) -> (f32, f32) {
    let (min, max) = match (min.is_finite(), max.is_finite()) {
        (true, true) => (min, max),
        // Grow around the surviving bound with next_up/next_down when a
        // fixed 1.0 span would round away at the f32 extremes.
        (true, false) => {
            let healed = if min + 1.0 > min {
                min + 1.0
            } else {
                min.next_up()
            };
            (min, healed)
        }
        (false, true) => {
            let healed = if max - 1.0 < max {
                max - 1.0
            } else {
                max.next_down()
            };
            (healed, max)
        }
        (false, false) => (0.0, 1.0),
    };
    if min <= max {
        (min, max)
    } else {
        (max, min)
    }
}

/// Percent knobs: NaN reads as empty (0%), everything else saturates
/// at the ends like a clamp (±∞ included).
pub(crate) const fn sanitize_percent(raw: f32) -> f32 {
    if raw.is_nan() {
        0.0
    } else if raw > 100.0 {
        100.0
    } else if raw < 0.0 {
        0.0
    } else {
        raw
    }
}

/// Signed rates (rotation speed): any finite value passes (negative
/// spins the other way); garbage keeps the current value.
pub(crate) const fn sanitize_rate(raw: f32, default: f32) -> f32 {
    crate::sanitize::finite_or(raw, default)
}

/// [`sanitize_percent`] for 0..=1 fractions.
pub(crate) const fn sanitize_fraction(raw: f32) -> f32 {
    if raw.is_nan() {
        0.0
    } else if raw > 1.0 {
        1.0
    } else if raw < 0.0 {
        0.0
    } else {
        raw
    }
}

/// Shared default for the plain-f32 width knobs whose constructors use
/// 200pt (context menu, dropdown menu, slider, range slider, menubar
/// trigger).
pub(crate) const DEFAULT_WIDTH: f32 = 200.0;

#[cfg(test)]
mod sanitize_tests {
    use super::*;

    #[test]
    fn finite_positive_passes_through() {
        assert_eq!(sanitize_width(150.0, 280.0), 150.0);
        assert_eq!(sanitize_opt_width(150.0), Some(150.0));
        assert_eq!(sanitize_height(28.0, 36.0), 28.0);
        assert_eq!(sanitize_opt_height(28.0), Some(28.0));
        assert_eq!(sanitize_size(13.0, 14.0), 13.0);
        assert_eq!(sanitize_opt_size(13.0), Some(13.0));
    }

    #[test]
    fn garbage_falls_back() {
        for garbage in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -5.0] {
            assert_eq!(sanitize_width(garbage, 280.0), 280.0);
            assert_eq!(sanitize_opt_width(garbage), None);
            assert_eq!(sanitize_height(garbage, 36.0), 36.0);
            assert_eq!(sanitize_opt_height(garbage), None);
            assert_eq!(sanitize_size(garbage, 14.0), 14.0);
            assert_eq!(sanitize_opt_size(garbage), None);
        }
    }

    /// Padding admits zero (flush); garbage falls back / counts as
    /// unset — pinning the sentinel classification at the wrapper level.
    #[test]
    fn padding_admits_zero_and_rejects_garbage() {
        assert_eq!(sanitize_padding(0.0, 8.0), 0.0);
        assert_eq!(sanitize_padding(4.0, 8.0), 4.0);
        assert_eq!(sanitize_opt_padding(0.0), Some(0.0));
        assert_eq!(sanitize_opt_padding(4.0), Some(4.0));
        for garbage in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.5] {
            assert_eq!(sanitize_padding(garbage, 8.0), 8.0);
            assert_eq!(sanitize_opt_padding(garbage), None);
        }
    }

    /// Radii admit zero (square) and cap where the u8 conversion
    /// saturates anyway; value-domain knobs accept any finite value.
    #[test]
    fn radius_and_value_policies() {
        assert_eq!(sanitize_radius(0.0, 6.0), 0.0);
        assert_eq!(sanitize_radius(8.0, 6.0), 8.0);
        assert_eq!(sanitize_radius(300.0, 6.0), 255.0);
        assert_eq!(sanitize_opt_radius(12.0), Some(12.0));
        assert_eq!(sanitize_opt_radius(f32::NAN), None);
        assert_eq!(sanitize_opt_value(-5.0), Some(-5.0));
        // Percent/fraction: NaN reads as empty, ±∞ saturate like clamp.
        assert_eq!(sanitize_percent(f32::NAN), 0.0);
        assert_eq!(sanitize_percent(f32::INFINITY), 100.0);
        assert_eq!(sanitize_percent(f32::NEG_INFINITY), 0.0);
        assert_eq!(sanitize_percent(60.0), 60.0);
        assert_eq!(sanitize_fraction(f32::NAN), 0.0);
        assert_eq!(sanitize_fraction(f32::INFINITY), 1.0);
        assert_eq!(sanitize_fraction(-0.5), 0.0);
        // Rates are signed: negative spins the other way.
        assert_eq!(sanitize_rate(-3.0, 1.0), -3.0);
        assert_eq!(sanitize_rate(f32::NAN, 1.0), 1.0);
        assert_eq!(sanitize_opt_value(f32::INFINITY), None);
        assert_eq!(sanitize_bounds(2.0, 8.0), (2.0, 8.0));
        assert_eq!(sanitize_bounds(8.0, 2.0), (2.0, 8.0));
        assert_eq!(sanitize_bounds(f32::NAN, f32::NAN), (0.0, 1.0));
        // One-sided garbage heals around the surviving bound — the
        // caller's explicit bound is never contradicted, and the healed
        // span survives IEEE rounding at the f32 extremes (a fixed
        // +1.0 would saturate back to a degenerate equal pair).
        assert_eq!(sanitize_bounds(5.0, f32::NAN), (5.0, 6.0));
        assert_eq!(sanitize_bounds(f32::NAN, 5.0), (4.0, 5.0));
        let (lo, hi) = sanitize_bounds(f32::MAX, f32::NAN);
        assert!(lo < hi, "span must grow past f32::MAX: {lo} .. {hi}");
        let (lo, hi) = sanitize_bounds(f32::NAN, f32::MIN);
        assert!(lo < hi, "span must grow below f32::MIN: {lo} .. {hi}");
    }
}

/// The knob policy is only as good as its coverage: every builder in
/// this module that takes a raw f32 must route it through a sanitize
/// wrapper (or sit on the explicit allowlist below). This test scans
/// the component sources so a new raw knob cannot slip in silently.
#[cfg(test)]
mod knob_inventory_tests {
    /// Builders deliberately raw, as `file: name` pairs with reasons:
    /// - `input.rs: font_size`: a no-op backwards-compat alias that
    ///   ignores its argument.
    /// - `sidebar.rs: set_widths`: delegates to the sanitizing
    ///   per-field setters.
    const ALLOWED_RAW: &[&str] = &["input.rs: font_size", "sidebar.rs: set_widths"];

    /// Strip comments so a body can only satisfy the scan by *calling*
    /// a sanitizer, not by mentioning one in prose. String-literal aware
    /// (`//` inside a string is code), nested block comments, and
    /// lifetimes (`'a`) are not treated as char literals.
    fn without_comments(src: &str) -> String {
        let bytes = src.as_bytes();
        let mut out = String::with_capacity(src.len());
        let mut i = 0usize;
        let mut in_string = false;
        let mut in_char = false;
        let mut block_depth = 0usize;
        while i < bytes.len() {
            let b = bytes[i];
            let next = bytes.get(i + 1).copied();
            if block_depth > 0 {
                if b == b'/' && next == Some(b'*') {
                    block_depth += 1;
                    i += 2;
                } else if b == b'*' && next == Some(b'/') {
                    block_depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            if in_string || in_char {
                out.push(b as char);
                if b == b'\\' {
                    if let Some(n) = next {
                        out.push(n as char);
                    }
                    i += 2;
                    continue;
                }
                if (in_string && b == b'"') || (in_char && b == b'\'') {
                    in_string = false;
                    in_char = false;
                }
                i += 1;
                continue;
            }
            match b {
                b'"' => {
                    in_string = true;
                    out.push(b as char);
                    i += 1;
                }
                b'\'' => {
                    // A char literal closes immediately ('x', '\n');
                    // a lifetime ('a) does not enter char state.
                    let is_char_literal = next == Some(b'\\') || bytes.get(i + 2) == Some(&b'\'');
                    if is_char_literal {
                        in_char = true;
                    }
                    out.push(b as char);
                    i += 1;
                }
                b'/' if next == Some(b'/') => {
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                }
                b'/' if next == Some(b'*') => {
                    block_depth = 1;
                    i += 2;
                }
                _ => {
                    out.push(b as char);
                    i += 1;
                }
            }
        }
        out
    }

    /// The stripper itself: prose in any comment form must not survive,
    /// and code after `//` inside a string literal must.
    #[test]
    fn comment_stripper_is_not_flatterable() {
        let src = r#"
/// sanitize_percent handles docs
pub const fn a(mut self, v: f32) -> Self {
    /* we sanitize_percent
       at the boundary */ self.a = v;
    let url = "https://example";
    let life: &'a str = "x"; // sanitize_width in a comment
    self.b = sanitize_width(v, self.b);
}
"#;
        let stripped = without_comments(src);
        assert!(!stripped.contains("sanitize_percent"), "{stripped}");
        assert!(!stripped.contains("in a comment"), "{stripped}");
        assert!(stripped.contains("https://example"), "{stripped}");
        assert!(stripped.contains("sanitize_width(v, self.b)"), "{stripped}");
    }

    #[test]
    fn every_f32_builder_sanitizes_or_is_allowlisted() {
        let dir = format!("{}/src/components", env!("CARGO_MANIFEST_DIR"));
        let mut offenders = Vec::new();
        let mut builders = 0usize;

        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let file = path.file_name().unwrap().to_string_lossy().to_string();
            if !file.to_lowercase().ends_with(".rs") || file == "mod.rs" {
                continue;
            }
            let src = without_comments(&std::fs::read_to_string(&path).unwrap());
            let mut i = 0usize;
            loop {
                let next = [src[i..].find("pub const fn "), src[i..].find("pub fn ")]
                    .into_iter()
                    .flatten()
                    .min();
                let Some(rel) = next else { break };
                let fn_start = i + rel;
                let name_start = fn_start + src[fn_start..].find("fn ").unwrap() + 3;
                let name_end = name_start + src[name_start..].find('(').unwrap();
                let name = src[name_start..name_end].to_string();
                let args_open = name_end;
                let args_close = args_open + src[args_open..].find(')').unwrap();
                let args = &src[args_open..args_close];

                // A bodyless signature (trait method, `;`-terminated)
                // must skip, not stop, the scan of this file.
                let brace = src[args_close..].find('{');
                let semi = src[args_close..].find(';');
                let body_open = match (brace, semi) {
                    (Some(b), Some(s)) if s < b => {
                        i = args_close + s + 1;
                        continue;
                    }
                    (Some(b), _) => args_close + b,
                    (None, _) => break,
                };
                let mut depth = 1usize;
                let mut j = body_open + 1;
                while depth > 0 && j < src.len() {
                    match src.as_bytes()[j] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                let body = &src[body_open..j];
                if args.contains(": f32") {
                    builders += 1;
                    let key = format!("{file}: {name}");
                    if !body.contains("sanitize_") && !ALLOWED_RAW.contains(&key.as_str()) {
                        offenders.push(key);
                    }
                }
                i = j;
            }
        }

        assert!(
            offenders.is_empty(),
            "raw f32 knobs without boundary sanitization:\n  {}",
            offenders.join("\n  ")
        );
        // The scan itself must stay meaningful.
        assert!(
            builders > 80,
            "only {builders} f32 builders found — did the scan break?"
        );
    }
}
