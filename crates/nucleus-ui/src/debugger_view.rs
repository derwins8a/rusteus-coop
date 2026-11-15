//! Handler step-by-step runtime debugger and variable inspector view.

use crate::components::badge::{CouchTag, TagTone};
use crate::components::button::{CouchButton, CouchButtonSize};
use crate::theme;
use gpui::prelude::*;
use gpui::*;
use gpui_kit_assets::IconName;
use nucleus_debug::protocol::{Breakpoint, DebugLaunchMode, PipelineStage};
use std::sync::Arc;

/// Presentation state for the debugger panel view.
#[derive(Debug, Clone)]
pub struct DebuggerViewState {
    pub is_attached: bool,
    pub is_paused: bool,
    pub current_file: String,
    pub current_line: u32,
    pub current_col: u32,
    pub active_stage: PipelineStage,
    pub launch_mode: DebugLaunchMode,
    pub breakpoints: Vec<Breakpoint>,
    pub call_stack: Vec<String>,
    pub inspector_variables: Vec<(String, String, String)>, // (Name, Type, Value)
    pub eval_input: String,
    pub eval_output: Option<String>,
    pub script_source_lines: Vec<String>,
}

impl Default for DebuggerViewState {
    fn default() -> Self {
        Self {
            is_attached: false,
            is_paused: false,
            current_file: "handler.js".to_string(),
            current_line: 1,
            current_col: 1,
            active_stage: PipelineStage::ScriptEvaluate,
            launch_mode: DebugLaunchMode::Normal,
            breakpoints: Vec::new(),
            call_stack: vec!["<top-level>".to_string()],
            inspector_variables: Vec::new(),
            eval_input: String::new(),
            eval_output: None,
            script_source_lines: Vec::new(),
        }
    }
}

/// Renders the complete Handler Runtime Debugger & Inspector panel.
pub fn render_debugger_view(
    state: &DebuggerViewState,
    on_resume: Option<Arc<dyn Fn() + Send + Sync>>,
    on_step_over: Option<Arc<dyn Fn() + Send + Sync>>,
    on_step_into: Option<Arc<dyn Fn() + Send + Sync>>,
    on_step_out: Option<Arc<dyn Fn() + Send + Sync>>,
    on_pause: Option<Arc<dyn Fn() + Send + Sync>>,
    on_stop: Option<Arc<dyn Fn() + Send + Sync>>,
    on_eval: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    on_toggle_bp: Option<Arc<dyn Fn(&str, u32) + Send + Sync>>,
) -> AnyElement {
    let resume_cb = on_resume.clone();
    let step_over_cb = on_step_over.clone();
    let step_into_cb = on_step_into.clone();
    let step_out_cb = on_step_out.clone();
    let pause_cb = on_pause.clone();
    let stop_cb = on_stop.clone();
    let eval_cb = on_eval.clone();
    let eval_query = state.eval_input.clone();

    div()
        .flex()
        .flex_col()
        .size_full()
        .p_6()
        .gap_4()
        .bg(theme::bg_main())
        .child(
            // Top Debugger Toolbar
            div()
                .flex()
                .items_center()
                .justify_between()
                .p_3()
                .rounded_lg()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_color())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Handler Debugger"),
                        )
                        .child(
                            CouchTag::new(if state.is_paused { "Paused" } else { "Running" })
                                .tone(if state.is_paused { TagTone::Warning } else { TagTone::Success })
                                .icon(if state.is_paused { IconName::Pause } else { IconName::Play }),
                        )
                        .child(
                            CouchTag::new(match state.launch_mode {
                                DebugLaunchMode::Normal => "Normal Mode",
                                DebugLaunchMode::DryRun => "Dry-Run Simulation",
                                DebugLaunchMode::SingleInstance => "Single-Instance Mode",
                                DebugLaunchMode::VirtualGamepads => "Virtual Gamepads",
                            })
                            .tone(TagTone::Neutral),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            CouchButton::primary("btn-dbg-resume", "Continue")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::Play)
                                .disabled(!state.is_paused)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = resume_cb {
                                        cb();
                                    }
                                }),
                        )
                        .child(
                            CouchButton::secondary("btn-dbg-step-over", "Step Over")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::ChevronRight)
                                .disabled(!state.is_paused)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = step_over_cb {
                                        cb();
                                    }
                                }),
                        )
                        .child(
                            CouchButton::secondary("btn-dbg-step-into", "Step Into")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::ChevronDown)
                                .disabled(!state.is_paused)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = step_into_cb {
                                        cb();
                                    }
                                }),
                        )
                        .child(
                            CouchButton::secondary("btn-dbg-step-out", "Step Out")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::ChevronUp)
                                .disabled(!state.is_paused)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = step_out_cb {
                                        cb();
                                    }
                                }),
                        )
                        .child(
                            CouchButton::secondary("btn-dbg-pause", "Pause")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::Pause)
                                .disabled(state.is_paused)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = pause_cb {
                                        cb();
                                    }
                                }),
                        )
                        .child(
                            CouchButton::danger("btn-dbg-stop", "Stop")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::X)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = stop_cb {
                                        cb();
                                    }
                                }),
                        ),
                ),
        )
        .child(
            // Pipeline Stage Stepper Breadcrumb Track
            div()
                .id("dbg-pipeline-stages-scroll")
                .flex()
                .items_center()
                .gap_2()
                .p_2p5()
                .rounded_md()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_color())
                .overflow_x_scroll()
                .children([
                    PipelineStage::ScriptEvaluate,
                    PipelineStage::OptionsUi,
                    PipelineStage::PrepareInstances,
                    PipelineStage::MutexKillSweep,
                    PipelineStage::ProcessSpawn,
                    PipelineStage::HwndPoll,
                    PipelineStage::ApplyBounds,
                    PipelineStage::HeartbeatTick,
                    PipelineStage::Teardown,
                ].iter().map(|stage| {
                    let is_current = *stage == state.active_stage;
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            CouchTag::new(stage.display_label())
                                .tone(if is_current { TagTone::Accent } else { TagTone::Neutral })
                                .icon(if is_current { IconName::CircleCheck } else { IconName::Clock }),
                        )
                        .into_any_element()
                })),
        )
        .child(
            // Main Debugger Workspace Split
            div()
                .flex()
                .size_full()
                .gap_4()
                .child(
                    // Left Column: Breakpoints & Call Stack
                    div()
                        .flex()
                        .flex_col()
                        .w(px(240.0))
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_3()
                                .rounded_md()
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(theme::border_color())
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Call Stack"),
                                )
                                .children(state.call_stack.iter().map(|frame| {
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(format!("• {}", frame))
                                        .into_any_element()
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_3()
                                .rounded_md()
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(theme::border_color())
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Breakpoints"),
                                 )
                                .children(if state.breakpoints.is_empty() {
                                    vec![div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child("No breakpoints set")
                                        .into_any_element()]
                                } else {
                                    state.breakpoints.iter().map(|bp| {
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .text_xs()
                                            .text_color(theme::text_primary())
                                            .child(format!("{}:{}", bp.file_id, bp.line))
                                            .child(CouchTag::new("Active").tone(TagTone::Success))
                                            .into_any_element()
                                    }).collect()
                                }),
                        ),
                )
                .child(
                    // Center Column: Source Code View with Line Highlight
                    div()
                        .id("dbg-source-code-scroll")
                        .flex()
                        .flex_col()
                        .flex_1()
                        .p_3()
                        .rounded_md()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_color())
                        .overflow_y_scroll()
                        .children(if state.script_source_lines.is_empty() {
                            vec![div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size_full()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("No handler script source loaded.")
                                .into_any_element()]
                        } else {
                            state.script_source_lines.iter().enumerate().map(|(idx, line)| {
                                let line_no = (idx + 1) as u32;
                                let is_current_line = line_no == state.current_line;
                                let has_bp = state.breakpoints.iter().any(|b| b.line == line_no);
                                let toggle_cb = on_toggle_bp.clone();
                                let file_id = state.current_file.clone();

                                div()
                                    .id(ElementId::Name(format!("code-line-{}", line_no).into()))
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_sm()
                                    .cursor_pointer()
                                    .on_click(move |_ev, _win, _app| {
                                        if let Some(ref cb) = toggle_cb {
                                            cb(&file_id, line_no);
                                        }
                                    })
                                    .bg(if is_current_line {
                                        theme::accent_subtle()
                                    } else {
                                        theme::bg_panel()
                                    })
                                    .child(
                                        div()
                                            .w(px(16.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(if has_bp {
                                                div()
                                                    .size(px(8.0))
                                                    .rounded_full()
                                                    .bg(theme::danger())
                                                    .into_any_element()
                                            } else if is_current_line {
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::accent())
                                                    .child("▶")
                                                    .into_any_element()
                                            } else {
                                                div().into_any_element()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .w(px(32.0))
                                            .text_right()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme::text_muted())
                                            .child(line_no.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if is_current_line {
                                                theme::accent()
                                            } else {
                                                theme::text_primary()
                                            })
                                            .child(line.clone()),
                                    )
                                    .into_any_element()
                            }).collect()
                        }),
                )
                .child(
                    // Right Column: Variable Inspector Tree
                    div()
                        .flex()
                        .flex_col()
                        .w(px(300.0))
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_3()
                                .rounded_md()
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(theme::border_color())
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Variable Inspector"),
                                )
                                .children(if state.inspector_variables.is_empty() {
                                    vec![div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child("Pause execution to inspect live JS objects and context.")
                                        .into_any_element()]
                                } else {
                                    state.inspector_variables.iter().map(|(name, var_type, val)| {
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .text_xs()
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(theme::accent())
                                                    .child(name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_color(theme::text_muted())
                                                    .child(format!("({}) {}", var_type, val)),
                                            )
                                            .into_any_element()
                                    }).collect()
                                }),
                        )
                        .child(
                            // Debug REPL / Expression Evaluator
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_3()
                                .rounded_md()
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(theme::border_color())
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Console / Eval REPL"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .flex_1()
                                                .h(px(28.0))
                                                .px_2()
                                                .rounded_md()
                                                .bg(theme::bg_main())
                                                .border_1()
                                                .border_color(theme::border_color())
                                                .text_xs()
                                                .text_color(theme::text_primary())
                                                .child(if eval_query.is_empty() {
                                                    "Eval JS expr (e.g. Game.ExecutableName)...".to_string()
                                                } else {
                                                    eval_query.clone()
                                                }),
                                        )
                                        .child(
                                            CouchButton::primary("btn-dbg-eval", "Eval")
                                                .size(CouchButtonSize::Sm)
                                                .on_click(move |_ev, _win, _app| {
                                                    if let Some(ref cb) = eval_cb {
                                                        cb(&eval_query);
                                                    }
                                                }),
                                        ),
                                )
                                .when_some(state.eval_output.as_ref(), |this, out| {
                                    this.child(
                                        div()
                                            .p_2()
                                            .rounded_sm()
                                            .bg(theme::bg_main())
                                            .text_xs()
                                            .text_color(theme::text_muted())
                                            .child(out.clone()),
                                    )
                                }),
                        ),
                ),
        )
        .into_any_element()
}
