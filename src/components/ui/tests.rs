#![allow(unused_imports)]
use dioxus::prelude::*;
use crate::components::ui::accordion::*;
use crate::components::ui::alert::*;
use crate::components::ui::alert_dialog::*;
use crate::components::ui::aspect_ratio::*;
use crate::components::ui::breadcrumb::*;
use crate::components::ui::checkbox::*;
use crate::components::ui::hover_card::*;
use crate::components::ui::input_otp::*;
use crate::components::ui::label::*;
use crate::components::ui::menubar::*;
use crate::components::ui::navigation_menu::*;
use crate::components::ui::pagination::*;
use crate::components::ui::popover::*;
use crate::components::ui::progress::*;
use crate::components::ui::radio_group::*;
use crate::components::ui::scroll_area::*;
use crate::components::ui::sheet::*;
use crate::components::ui::skeleton::*;
use crate::components::ui::slider::*;
use crate::components::ui::table::*;
use crate::components::ui::textarea::*;
use crate::components::ui::toggle::*;
use crate::components::ui::toggle_group::*;

#[test]
fn test_alert_variants() {
    assert_eq!(AlertVariant::from_str_name("destructive"), AlertVariant::Destructive);
    assert_eq!(AlertVariant::from_str_name("warning"), AlertVariant::Warning);
    assert_eq!(AlertVariant::from_str_name("success"), AlertVariant::Success);
    assert_eq!(AlertVariant::from_str_name("info"), AlertVariant::Info);
    assert_eq!(AlertVariant::from_str_name("default"), AlertVariant::Default);

    let dest_cls = AlertVariant::Destructive.class_name();
    assert!(dest_cls.contains("text-destructive"));
}

#[test]
fn test_toggle_variants_and_sizes() {
    assert_eq!(ToggleVariant::default(), ToggleVariant::Default);
    assert!(ToggleVariant::Outline.class_name().contains("border-input"));

    assert_eq!(ToggleSize::default(), ToggleSize::Default);
    assert!(ToggleSize::Sm.class_name().contains("h-8"));
    assert!(ToggleSize::Lg.class_name().contains("h-10"));
}

#[test]
fn test_aspect_ratio_math() {
    let default_ratio = 16.0 / 9.0;
    assert!(default_ratio > 1.7 && default_ratio < 1.8);
}

#[test]
fn test_progress_clamping() {
    let val = 120.0f64;
    let max = 100.0f64;
    let clamped = val.clamp(0.0, max);
    assert_eq!(clamped, 100.0);

    let pct = (clamped / max) * 100.0;
    assert_eq!(pct, 100.0);
}

#[component]
fn ShadcnCatalogTestApp() -> Element {
    let radio_val = use_signal(|| "option-1".to_string());
    let slider_val = use_signal(|| 45.0f64);
    let toggle_group_val = use_signal(|| "b".to_string());
    let otp_val = use_signal(|| "123".to_string());
    let acc_val = use_signal(|| vec!["item-1".to_string()]);

    rsx! {
        div { class: "p-4 space-y-4",
            // Alert
            Alert { variant: "info",
                AlertTitle { "Notification" }
                AlertDescription { "Everything is running smoothly." }
            }

            // Label & Textarea
            Label { r#for: "notes", required: true, "Notes" }
            Textarea { id: "notes", placeholder: "Type here...", rows: 4 }

            // Checkbox
            Checkbox { checked: true }

            // Progress
            Progress { value: 65.0, max: 100.0 }

            // Skeleton
            Skeleton { class: "h-6 w-32" }

            // Aspect Ratio
            AspectRatio { ratio: 16.0 / 9.0,
                div { class: "bg-muted size-full" }
            }

            // Scroll Area
            ScrollArea { class: "h-32",
                div { class: "p-2", "Scrollable text" }
                ScrollBar {}
            }

            // Radio Group
            RadioGroup { value: radio_val,
                RadioGroupItem { value: "option-1" }
                RadioGroupItem { value: "option-2" }
            }

            // Slider
            Slider { value: slider_val, min: 0.0, max: 100.0 }

            // Toggle & Toggle Group
            Toggle { pressed: true, "Bold" }
            ToggleGroup { value: toggle_group_val,
                ToggleGroupItem { value: "a", "A" }
                ToggleGroupItem { value: "b", "B" }
            }

            // Input OTP
            InputOTP { value: otp_val, max_length: 6,
                InputOTPGroup {
                    InputOTPSlot { index: 0 }
                    InputOTPSlot { index: 1 }
                    InputOTPSlot { index: 2 }
                }
                InputOTPSeparator {}
                InputOTPGroup {
                    InputOTPSlot { index: 3 }
                    InputOTPSlot { index: 4 }
                    InputOTPSlot { index: 5 }
                }
            }

            // Popover
            Popover {
                PopoverTrigger { "Open Popover" }
                PopoverContent { "Popover Content" }
            }

            // Hover Card
            HoverCard {
                HoverCardTrigger { "Hover over me" }
                HoverCardContent { "Hover Card details" }
            }

            // Alert Dialog
            AlertDialog {
                AlertDialogTrigger { "Delete" }
                AlertDialogContent {
                    AlertDialogHeader {
                        AlertDialogTitle { "Are you sure?" }
                        AlertDialogDescription { "This action cannot be undone." }
                    }
                    AlertDialogFooter {
                        AlertDialogCancel { "Cancel" }
                        AlertDialogAction { "Continue" }
                    }
                }
            }

            // Sheet
            Sheet {
                SheetTrigger { "Open Sheet" }
                SheetContent { side: "right",
                    SheetHeader {
                        SheetTitle { "Edit Profile" }
                        SheetDescription { "Make changes to your profile." }
                    }
                    SheetFooter {
                        SheetClose { "Save Changes" }
                    }
                }
            }

            // Accordion
            Accordion { r#type: "single", value: acc_val,
                AccordionItem { value: "item-1",
                    AccordionTrigger { "Is it accessible?" }
                    AccordionContent { "Yes. It adheres to WAI-ARIA design patterns." }
                }
            }

            // Breadcrumb
            Breadcrumb {
                BreadcrumbList {
                    BreadcrumbItem { BreadcrumbLink { href: "/", "Home" } }
                    BreadcrumbSeparator {}
                    BreadcrumbItem { BreadcrumbLink { href: "/docs", "Docs" } }
                    BreadcrumbSeparator {}
                    BreadcrumbItem { BreadcrumbPage { "Components" } }
                }
            }

            // Pagination
            Pagination {
                PaginationContent {
                    PaginationItem { PaginationPrevious {} }
                    PaginationItem { PaginationLink { is_active: true, "1" } }
                    PaginationItem { PaginationEllipsis {} }
                    PaginationItem { PaginationNext {} }
                }
            }

            // Table
            Table {
                TableHeader {
                    TableRow {
                        TableHead { "Name" }
                        TableHead { "Status" }
                    }
                }
                TableBody {
                    TableRow {
                        TableCell { "Project A" }
                        TableCell { "Active" }
                    }
                }
                TableFooter {
                    TableRow {
                        TableCell { "Total" }
                        TableCell { "1" }
                    }
                }
            }

            // Menubar
            Menubar {
                MenubarMenu { id: "file",
                    MenubarTrigger { "File" }
                    MenubarContent {
                        MenubarItem { "New Tab" }
                        MenubarSeparator {}
                        MenubarItem { "Print" }
                    }
                }
            }

            // Navigation Menu
            NavigationMenu {
                NavigationMenuList {
                    NavigationMenuItem { id: "item-1",
                        NavigationMenuTrigger { "Getting Started" }
                        NavigationMenuContent {
                            NavigationMenuLink { href: "/intro", "Introduction" }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_all_shadcn_components_instantiate_in_vdom() {
    let mut dom = VirtualDom::new(ShadcnCatalogTestApp);
    dom.rebuild_in_place();
}

#[test]
fn test_avatar_fallback_instantiation() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            crate::components::ui::avatar::Avatar {
                crate::components::ui::avatar::AvatarImage { src: "".to_string(), alt: "User".to_string() }
                crate::components::ui::avatar::AvatarFallback { "JD" }
            }
        }
    });
    dom.rebuild_in_place();
}
