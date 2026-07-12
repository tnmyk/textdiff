# TextDiff Tool Implementation Plan

Based on our design discussion, here is the technical plan for building your online long text comparison tool:

## 1. Core Engine (Diffing Algorithm)
- **Library**: We will use the `similar` crate. It is a highly performant Rust diffing library that seamlessly compiles to WebAssembly.
- **Granularity**: The engine will be configured to perform a line-by-line comparison first, and then run a word-level diff on modified lines. This provides the best UX by showing exactly which words changed in a long paragraph.

## 2. Application State & Modes
The application will have two primary states, toggled via a button in a top toolbar:
- **Edit Mode**: The default state where users input their text.
- **Diff Mode**: The state where the comparison is visualized.

## 3. User Interface (egui)

### Edit Mode UI
- A side-by-side layout utilizing `egui::Columns`.
- Two large `TextEdit::multiline` input areas (Left for "Original", Right for "Modified").
- Optimized to take up the full available screen space.

### Diff Mode UI (Read-Only)
- A side-by-side layout presenting the diff results.
- **Synchronized Scrolling**: We will use `egui::ScrollArea` and link the scroll offsets so scrolling one side automatically scrolls the other.
- **Visual Highlighting**:
  - Unchanged lines: Default text color.
  - Deleted lines (Left side): Light red background.
  - Added lines (Right side): Light green background.
  - *Inline word changes*: Darker red/green backgrounds applied to the specific words that changed within a modified line.
- **Formatting**: Plain text rendering (no syntax highlighting) to ensure maximum performance for very long text and unstructured prose.

## 4. Next Steps for Implementation
1. Add the `similar` crate to `Cargo.toml`.
2. Update `src/app.rs` to include the `mode` state (Edit vs Diff) and the two text buffers.
3. Build the `Edit Mode` UI.
4. Implement the diff calculation using `similar` and build the `Diff Mode` UI with synchronized scrolling and colored highlights.
