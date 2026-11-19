//! Activities Cadence Chart.
//!
//! Clean flat-bar visualization with y-axis grid lines, top-edge bar shadows,
//! and responsive SVG viewBox. Built with shadcn UI components.

use dioxus::prelude::*;
use crate::components::ui::card::{Card, CardTitle};
use crate::components::ui::select::{Select, SelectContent, SelectItem, SelectTrigger, SelectValue};

#[derive(Props, Clone, PartialEq)]
pub struct ActivitiesChartProps {
    pub days_activity: Vec<(&'static str, usize)>,
    pub max_axis: usize,
    pub activity_ticks: Vec<usize>,
    pub activity_timeframe: Signal<String>,
}

#[component]
pub fn ActivitiesChart(props: ActivitiesChartProps) -> Element {
    let timeframe = props.activity_timeframe;
    let mut hovered_index = use_signal(|| None::<usize>);

    let days = &props.days_activity;
    let max_val = props.max_axis.max(1);

    // SVG Layout parameters
    let chart_w: f64 = 540.0;
    let chart_h: f64 = 280.0;
    let plot_x: f64 = 52.0;
    let plot_w: f64 = chart_w - plot_x - 20.0; // 468.0
    let plot_top: f64 = 20.0;
    let plot_h: f64 = 200.0;
    let plot_bottom: f64 = plot_top + plot_h; // 220.0
    let bar_w: f64 = 38.0;
    let bar_rx: f64 = 6.0;

    let num_days = days.len().max(1);
    let col_step = plot_w / (num_days as f64);

    let num_ticks = props.activity_ticks.len().max(2);
    let tick_step_h = plot_h / ((num_ticks - 1) as f64);

    rsx! {
        Card {
            class: "flex flex-col justify-between transition-all duration-200 hover:shadow-sm overflow-visible",

            // Card Header: Title on left + shadcn Select dropdown on right
            div { class: "flex items-center justify-between p-6 pb-2 relative z-20",
                CardTitle { class: "text-lg font-bold tracking-tight", "Activities" }

                // Shadcn UI Select
                div { class: "w-[125px] relative z-50", style: "z-index: 50;",
                    Select {
                        class: "z-50",
                        value: Some(timeframe),
                        placeholder: "this week".to_string(),
                        SelectTrigger {
                            class: "h-8 px-3 text-xs font-semibold text-muted-foreground rounded-full border-border/80 bg-background hover:text-foreground",
                            SelectValue { placeholder: "this week".to_string() }
                        }
                        SelectContent {
                            SelectItem { value: "this_week".to_string(), text: "this week".to_string(), "this week" }
                            SelectItem { value: "last_week".to_string(), text: "last week".to_string(), "last week" }
                            SelectItem { value: "all_time".to_string(), text: "all time".to_string(), "all time" }
                        }
                    }
                }
            }

            // High Precision SVG Chart
            div { class: "relative w-full overflow-hidden px-6 pb-6 pt-0 mt-1",
                svg {
                    view_box: "0 0 {chart_w} {chart_h}",
                    class: "w-full h-auto select-none overflow-visible",
                    preserve_aspect_ratio: "xMidYMid meet",

                    // Definitions: Gradients and Clip Paths
                    defs {
                        // Linear gradient for top-edge drop shadow
                        linearGradient {
                            id: "top-shadow-gradient",
                            x1: "0%",
                            y1: "0%",
                            x2: "0%",
                            y2: "100%",
                            stop { offset: "0%", stop_color: "rgba(0, 0, 0, 0.45)" }
                            stop { offset: "100%", stop_color: "rgba(0, 0, 0, 0.0)" }
                        }

                        // Clip paths for each column with gentle rounded corners
                        for (i, _) in days.iter().enumerate() {
                            {
                                let cx = plot_x + (i as f64 + 0.5) * col_step;
                                let x = cx - (bar_w / 2.0);
                                rsx! {
                                    clipPath {
                                        key: "clip-{i}",
                                        id: "bar-clip-{i}",
                                        rect {
                                            x: "{x}",
                                            y: "{plot_top}",
                                            width: "{bar_w}",
                                            height: "{plot_h}",
                                            rx: "{bar_rx}",
                                            ry: "{bar_rx}",
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 1. Horizontal Grid Lines & Y-Axis Labels
                    g { class: "grid-group",
                        for (idx, tick_val) in props.activity_ticks.iter().enumerate() {
                            {
                                let y = plot_top + (idx as f64) * tick_step_h;
                                rsx! {
                                    g { key: "grid-{idx}",
                                        // Numeric Y-axis tick label
                                        text {
                                            x: "{plot_x - 14.0}",
                                            y: "{y + 4.0}",
                                            text_anchor: "end",
                                            class: "fill-muted-foreground/75 text-[11px] font-semibold tabular-nums",
                                            "{tick_val}"
                                        }
                                        // Subtle horizontal background line
                                        line {
                                            x1: "{plot_x}",
                                            y1: "{y}",
                                            x2: "{plot_x + plot_w}",
                                            y2: "{y}",
                                            class: "stroke-border/40",
                                            stroke_width: "1",
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 2. 7 Subtly Rounded Bar Pillars
                    g { class: "bars-group",
                        for (i, (day_label, count)) in days.iter().enumerate() {
                            {
                                let cx = plot_x + (i as f64 + 0.5) * col_step;
                                let x = cx - (bar_w / 2.0);
                                let count_val = *count;
                                let fill_h = ((count_val as f64 / max_val as f64) * plot_h).clamp(0.0, plot_h);
                                let bar_y = plot_bottom - fill_h;
                                let is_hovered = hovered_index() == Some(i);

                                rsx! {
                                    g {
                                        key: "day-{day_label}",
                                        class: "bar-column cursor-pointer group",
                                        onmouseenter: move |_| hovered_index.set(Some(i)),
                                        onmouseleave: move |_| hovered_index.set(None),

                                        // Column hit-area for smooth hover interaction
                                        rect {
                                            x: "{cx - col_step / 2.0}",
                                            y: "{plot_top - 10.0}",
                                            width: "{col_step}",
                                            height: "{plot_h + 40.0}",
                                            fill: "transparent",
                                        }

                                        // Background Track: Subtly rounded rectangle
                                        rect {
                                            x: "{x}",
                                            y: "{plot_top}",
                                            width: "{bar_w}",
                                            height: "{plot_h}",
                                            rx: "{bar_rx}",
                                            ry: "{bar_rx}",
                                            class: if is_hovered {
                                                "fill-neutral-200/80 dark:fill-neutral-800 transition-colors duration-200"
                                            } else {
                                                "fill-neutral-100 dark:fill-neutral-800/60 transition-colors duration-200"
                                            },
                                        }

                                        // Active Fill Bar (gentle 6px rounded corners, perfectly bounded)
                                        if fill_h > 0.0 {
                                            // The active body
                                            rect {
                                                x: "{x}",
                                                y: "{bar_y}",
                                                width: "{bar_w}",
                                                height: "{fill_h + bar_rx}",
                                                rx: "{bar_rx}",
                                                ry: "{bar_rx}",
                                                class: "fill-[#212529] dark:fill-neutral-100 transition-all duration-300",
                                                clip_path: "url(#bar-clip-{i})",
                                            }

                                            // Shadow line at the top edge of the filled bar
                                            rect {
                                                x: "{x}",
                                                y: "{bar_y}",
                                                width: "{bar_w}",
                                                height: "6",
                                                rx: "{bar_rx}",
                                                ry: "{bar_rx}",
                                                fill: "url(#top-shadow-gradient)",
                                                clip_path: "url(#bar-clip-{i})",
                                            }

                                            // Crisp highlight edge
                                            rect {
                                                x: "{x}",
                                                y: "{bar_y}",
                                                width: "{bar_w}",
                                                height: "1.5",
                                                rx: "{bar_rx}",
                                                ry: "{bar_rx}",
                                                class: "fill-black/30 dark:fill-black/20",
                                                clip_path: "url(#bar-clip-{i})",
                                            }
                                        }

                                        // X-Axis Day of Week Label
                                        text {
                                            x: "{cx}",
                                            y: "{plot_bottom + 26.0}",
                                            text_anchor: "middle",
                                            class: if is_hovered {
                                                "fill-foreground text-[12px] font-bold select-none transition-colors"
                                            } else {
                                                "fill-muted-foreground text-[12px] font-medium select-none transition-colors"
                                            },
                                            "{day_label}"
                                        }

                                        // Interactive hover count indicator badge above the bar
                                        if is_hovered {
                                            g { class: "tooltip-badge animate-in fade-in zoom-in-95 duration-150",
                                                rect {
                                                    x: "{cx - 24.0}",
                                                    y: "{plot_top - 18.0}",
                                                    width: "48",
                                                    height: "20",
                                                    rx: "4",
                                                    ry: "4",
                                                    class: "fill-foreground shadow-md",
                                                }
                                                text {
                                                    x: "{cx}",
                                                    y: "{plot_top - 5.0}",
                                                    text_anchor: "middle",
                                                    class: "fill-background text-[11px] font-black tabular-nums select-none",
                                                    "{count_val} acts"
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
