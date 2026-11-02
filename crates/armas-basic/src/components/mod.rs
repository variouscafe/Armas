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

/// Padding and spacing knobs, where zero is legal (flush / no gap): a
/// finite non-negative value passes, garbage falls back to `default`.
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

/// Shared default for the plain-f32 width knobs whose constructors use
/// 200pt (context menu, dropdown menu, slider, range slider).
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
}
