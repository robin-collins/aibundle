#!/bin/bash
base_dir="/home/tech/projects/aibundle-modular"
current_dir=$(pwd)

if [ "$current_dir" != "$base_dir" ]; then
    cd $base_dir || exit 1
fi

if [ ! -d "test_results" ]; then
    mkdir -p test_results
else
    rm -rf test_results/*
fi

echo "Running tests..."

CARGO_TERM_COLOR=never cargo test --doc src/cli/mod.rs > test_results/src_cli_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/cli/options.rs > test_results/src_cli_options_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/clipboard/mod.rs > test_results/src_clipboard_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/config/mod.rs > test_results/src_config_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/fs/mod.rs > test_results/src_fs_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/lib.rs > test_results/src_lib_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/models/app_config.rs > test_results/src_models_app_config_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/models/constants.rs > test_results/src_models_constants_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/models/enums.rs > test_results/src_models_enums_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/models/mod.rs > test_results/src_models_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/output/format.rs > test_results/src_output_format_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/output/json.rs > test_results/src_output_json_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/output/llm.rs > test_results/src_output_llm_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/output/markdown.rs > test_results/src_output_markdown_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/output/mod.rs > test_results/src_output_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/output/xml.rs > test_results/src_output_xml_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/app.rs > test_results/src_tui_app_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/components/file_list.rs > test_results/src_tui_components_file_list_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/components/header.rs > test_results/src_tui_components_header_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/components/mod.rs > test_results/src_tui_components_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/components/modal.rs > test_results/src_tui_components_modal_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/components/status_bar.rs > test_results/src_tui_components_status_bar_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/handlers/clipboard.rs > test_results/src_tui_handlers_clipboard_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/handlers/file_ops.rs > test_results/src_tui_handlers_file_ops_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/handlers/keyboard.rs > test_results/src_tui_handlers_keyboard_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/handlers/mod.rs > test_results/src_tui_handlers_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/handlers/search.rs > test_results/src_tui_handlers_search_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/mod.rs > test_results/src_tui_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/state/app_state.rs > test_results/src_tui_state_app_state_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/state/mod.rs > test_results/src_tui_state_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/state/search.rs > test_results/src_tui_state_search_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/state/selection.rs > test_results/src_tui_state_selection_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/views/help_view.rs > test_results/src_tui_views_help_view_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/views/main_view.rs > test_results/src_tui_views_main_view_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/views/message_view.rs > test_results/src_tui_views_message_view_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/tui/views/mod.rs > test_results/src_tui_views_mod_rs.txt
CARGO_TERM_COLOR=never cargo test --doc src/utils/mod.rs > test_results/src_utils_mod_rs.txt

if [ "$current_dir" != "$base_dir" ]; then
    cd $current_dir || exit 1
fi

echo "Tests completed. Results saved in test_results directory."