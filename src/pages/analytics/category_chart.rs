//! Issue Category Breakdown Chart.
//!
//! Features an interactive SVG donut distribution chart paired with ranked
//! category progress bars and volume metrics. Built with shadcn UI components.

use dioxus::prelude::*;
use crate::components::ui::badge::Badge;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::icons::TicketIcon;
use super::types::CategoryStat;

pub const CATEGORY_PALETTE: &[(&str, &str, &str)] = &[
    ("#3b82f6", "bg-blue-500", "text-blue-500"),       // Blue
    ("#10b981", "bg-emerald-500", "text-emerald-500"), // Emerald
    ("#f59e0b", "bg-amber-500", "text-amber-500"),     // Amber
    ("#8b5cf6", "bg-purple-500", "text-purple-500"),   // Purple
    ("#ec4899", "bg-pink-500", "text-pink-500"),       // Pink
    ("#06b6d4", "bg-cyan-500", "text-cyan-500"),       // Cyan
    ("#f97316", "bg-orange-500", "text-orange-500"),   // Orange
    ("#6366f1", "bg-indigo-500", "text-indigo-500"),   // Indigo
];

#[derive(Props, Clone, PartialEq)]
pub struct CategoryChartProps {
    pub category_stats: Vec<CategoryStat>,
    pub total_count: usize,
}

#[component]
pub fn CategoryChart(props: CategoryChartProps) -> Element {
    let mut hovered_cat = use_signal(|| None::<String>);

    let total = props.total_count;
    let categories = &props.category_stats;

    // SVG Donut geometry
    let donut_r: f64 = 60.0;
    let stroke_w: f64 = 17.0;
    let circumference = 2.0 * std::f64::consts::PI * donut_r; // ~376.99

    // Compute segment offsets with clean angular gaps
    let num_cats = categories.len();
    let gap: f64 = if num_cats > 1 { 3.5 } else { 0.0 };

    let mut segments: Vec<(&CategoryStat, f64, f64, &str)> = Vec::new();
    let mut current_offset = 0.0;
    for (i, cat) in categories.iter().enumerate() {
        let fraction = if total > 0 { cat.count as f64 / total as f64 } else { 0.0 };
        let stroke_len = fraction * circumference;
        let color = CATEGORY_PALETTE[i % CATEGORY_PALETTE.len()].0;
        segments.push((cat, stroke_len, current_offset, color));
        current_offset += stroke_len;
    }

    rsx! {
        Card {
            class: "flex flex-col justify-between transition-all duration-200 hover:shadow-sm overflow-hidden",

            // Header
            CardHeader {
                class: "flex flex-row items-center justify-between border-b border-border/60 p-6 pb-4",
                div { class: "flex items-center gap-2.5",
                    div { class: "flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary border border-primary/20",
                        TicketIcon { class: "size-4" }
                    }
                    div { class: "flex flex-col gap-0.5",
                        CardTitle { class: "text-sm font-bold text-foreground", "Issue Category Breakdown" }
                        p { class: "text-[11px] text-muted-foreground", "Ticket distribution by subject area" }
                    }
                }
                Badge {
                    variant: "secondary",
                    class: "font-mono text-xs",
                    "{categories.len()} categories"
                }
            }

            CardContent {
                class: "p-6 pt-5 flex-1 flex flex-col justify-center",
                if categories.is_empty() {
                    div { class: "py-14 text-center text-xs text-muted-foreground",
                        "No category records found for the selected filter."
                    }
                } else {
                    div { class: "grid grid-cols-1 md:grid-cols-12 gap-6 items-center",

                        // Left Column: Crisp Vector Donut Visualizer
                        div { class: "md:col-span-5 flex flex-col items-center justify-center relative",
                            div { class: "relative size-44 sm:size-48 flex items-center justify-center",
                                svg {
                                    view_box: "0 0 160 160",
                                    class: "size-full -rotate-90 transform overflow-visible select-none",

                                    // Background base track circle
                                    circle {
                                        cx: "80",
                                        cy: "80",
                                        r: "{donut_r}",
                                        fill: "none",
                                        stroke_width: "{stroke_w}",
                                        class: "stroke-muted/30",
                                    }

                                    // Segment strokes with clean butt-caps and crisp gaps
                                    for (stat, stroke_len, offset, color) in segments.iter() {
                                        {
                                            let is_active = hovered_cat().as_ref() == Some(&stat.name);
                                            let stroke_visible = (*stroke_len - gap).max(0.5);
                                            let dash_array = format!("{stroke_visible:.2} {circumference:.2}");
                                            let dash_offset = format!("-{:.2}", offset + gap / 2.0);
                                            rsx! {
                                                circle {
                                                    key: "{stat.name}",
                                                    cx: "80",
                                                    cy: "80",
                                                    r: "{donut_r}",
                                                    fill: "none",
                                                    stroke: "{color}",
                                                    stroke_width: if is_active { stroke_w + 3.5 } else { stroke_w },
                                                    stroke_dasharray: "{dash_array}",
                                                    stroke_dashoffset: "{dash_offset}",
                                                    stroke_linecap: "butt",
                                                    class: "transition-all duration-200 cursor-pointer hover:opacity-95",
                                                    onmouseenter: {
                                                        let name = stat.name.clone();
                                                        move |_| hovered_cat.set(Some(name.clone()))
                                                    },
                                                    onmouseleave: move |_| hovered_cat.set(None),
                                                }
                                            }
                                        }
                                    }
                                }

                                // Center Counter / Live Hover Insights
                                div { class: "pointer-events-none absolute inset-0 flex flex-col items-center justify-center text-center px-4",
                                    if let Some(h_name) = hovered_cat() {
                                        if let Some(s) = categories.iter().find(|c| c.name == h_name) {
                                            span { class: "text-3xl font-black tracking-tight text-foreground tabular-nums", "{s.count}" }
                                            span { class: "text-xs font-bold text-foreground/90 truncate max-w-[110px]", "{s.name}" }
                                            span { class: "text-[11px] font-bold text-primary mt-0.5", "{s.percentage:.1}% share" }
                                        }
                                    } else {
                                        span { class: "text-3xl font-black tracking-tight text-foreground tabular-nums", "{total}" }
                                        span { class: "text-[10px] font-extrabold uppercase tracking-widest text-muted-foreground", "TICKETS" }
                                        span { class: "text-[10px] text-muted-foreground/75 font-medium", "Total Active" }
                                    }
                                }
                            }
                        }

                        // Right Column: Clean Ranked Category Items (No ugly scrollbars)
                        div { class: "md:col-span-7 flex flex-col gap-2 pr-0.5",
                            for (i, cat) in categories.iter().enumerate() {
                                {
                                    let color_hex = CATEGORY_PALETTE[i % CATEGORY_PALETTE.len()].0;
                                    let is_active = hovered_cat().as_ref() == Some(&cat.name);
                                    rsx! {
                                        div {
                                            key: "{cat.name}",
                                            class: if is_active {
                                                "flex flex-col gap-1.5 p-2 rounded-xl bg-accent/60 border border-border/80 transition-all cursor-pointer shadow-2xs"
                                            } else {
                                                "flex flex-col gap-1.5 p-2 rounded-xl hover:bg-muted/40 border border-transparent transition-all cursor-pointer"
                                            },
                                            onmouseenter: {
                                                let name = cat.name.clone();
                                                move |_| hovered_cat.set(Some(name.clone()))
                                            },
                                            onmouseleave: move |_| hovered_cat.set(None),

                                            div { class: "flex items-center justify-between text-xs",
                                                div { class: "flex items-center gap-2 min-w-0 pr-2",
                                                    span {
                                                        class: "size-2.5 shrink-0 rounded-full ring-2 ring-background shadow-2xs",
                                                        style: "background-color: {color_hex};",
                                                    }
                                                    span { class: "font-semibold text-foreground/90 truncate", "{cat.name}" }
                                                }
                                                div { class: "flex items-center gap-1.5 shrink-0 font-mono text-[11px]",
                                                    span { class: "font-bold text-foreground tabular-nums", "{cat.count}" }
                                                    span { class: "text-muted-foreground/80 font-medium", "({cat.percentage:.1}%)" }
                                                }
                                            }
                                            div { class: "h-2 w-full rounded-full bg-muted/70 overflow-hidden shadow-inner",
                                                div {
                                                    class: "h-full rounded-full transition-all duration-300",
                                                    style: "width: {cat.percentage}%; background-color: {color_hex};",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
