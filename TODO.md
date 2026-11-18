# Pending High-Value Features

Gap audit against [text-compare.com](https://text-compare.com/), Nov 2026.

Work top-down, one item per change. The GPUI web rewrite is in progress on the `gpui-web` branch; diff computation already lives in the UI-agnostic `src/diff.rs`.

- `[ ]` pending
- `[x]` done

## Queue

- [x] **1. Swap texts**
  - Exchange Original and Modified contents.
  - Drop any existing diff and stay in edit mode so stale results cannot be shown.
  - Add a swap control between the panes or in the action row.
  - Done when: contents swap exactly, including trailing newlines; no stale diff remains.

- [x] **2. Clear all**
  - Clear both panes, reset diff state, return to edit mode.
  - Disable the control when both panes are already empty.
  - Done when: one action leaves both panes empty and the app ready for new input.

- [x] **3. Statistics bar**
  - Count added, removed, changed, and unchanged lines after compare.
  - Show compact badges in the diff header.
  - Done when: counts derive from the diff ops; added + removed + unchanged lines match both inputs; changed counts replacement blocks.

- [ ] **4. Jump to previous/next difference**
  - Add previous/next controls and show position, e.g. `2 of 7`.
  - Scroll the active changed block into view. Wrap at the ends.
  - Bind `N` and `P` in diff mode only.
  - Done when: every changed block is reachable; focus stays in the diff; editing keys are unaffected.

- [ ] **5. Show only differences**
  - Add a diff-header toggle: `Only changes`.
  - Keep two context lines around each change; collapse longer unchanged runs.
  - Each collapsed block gets `Show N unchanged lines`.
  - Done when: both panes stay row-aligned; expanding a block does not shift the opposite pane out of sync.

- [ ] **6. Ignore case / whitespace / blank lines**
  - Add three toggles, independently combinable.
  - Compare normalized text but render the user's original text.
  - Lines equal under the active normalization render as unchanged.
  - Show active filters in the diff header.
  - Done when: ignored differences never produce a change; inline highlights map back to displayed text; UTF-8 input is safe.

- [ ] **7. Character-level inline highlighting**
  - Refine changed word spans down to changed characters.
  - Keep line alignment and the existing side-by-side layout.
  - Done when: a one-character edit highlights only that character; multi-byte UTF-8 and emoji do not panic or split glyphs.

- [ ] **8. Open and drag-drop files**
  - Add an open-file control per pane and accept drag-drop onto a pane.
  - Support UTF-8 text and code files. Reject binary files with a clear message.
  - Show file name and size after loading.
  - Native: file dialog. Web: file input plus browser drop-event bridge.
  - Done when: text loads into the correct pane on native and web; failures leave existing text untouched.

- [ ] **9. Keyboard shortcuts**
  - `Ctrl/Cmd+Enter` — compare.
  - `Ctrl+Alt+C` — compare (text-compare parity).
  - `Ctrl+Alt+S` — swap.
  - `Ctrl+Alt+R` — clear all.
  - `N` / `P` — next/previous difference.
  - Show shortcut hints in tooltips or the footer.
  - Done when: shortcuts do not fire while typing in a text field or during IME composition.

## Deferred

Not in the current high-value queue:

- Titles above each comparison
- PDF export
- DOCX / XLSX / PDF import
- Email or share-link export
- Accounts, ads, paywalls
- 10-language UI
- Optimized handling for inputs above 10 MB

## Already Covered

- Side-by-side diff with synchronized scrolling
- Line-level additions, deletions, and replacements
- Inline word-level highlighting
- Paste/type input, compare, back to edit
- Example text
- Native and WebAssembly builds
- Fully local processing, no upload, no account
