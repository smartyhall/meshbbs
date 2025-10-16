# Smart Quote Normalization Fix

## Problem Statement

macOS and some terminal emulators automatically convert ASCII straight quotes to Unicode smart/curly quotes. This causes parse errors in the trigger DSL parser, which only recognizes straight quotes.

### Affected Characters

| Character | Unicode | Bytes | Name |
|-----------|---------|-------|------|
| " | U+201C | [226, 128, 156] | LEFT DOUBLE QUOTATION MARK |
| " | U+201D | [226, 128, 157] | RIGHT DOUBLE QUOTATION MARK |
| ' | U+2018 | [226, 128, 152] | LEFT SINGLE QUOTATION MARK |
| ' | U+2019 | [226, 128, 153] | RIGHT SINGLE QUOTATION MARK |
| " | U+0022 | [34] | STRAIGHT DOUBLE QUOTE (required) |
| ' | U+0027 | [39] | STRAIGHT SINGLE QUOTE (required) |

## Error Manifestation

**User Input:**
```
@object edit teleport_stone trigger OnUse teleport("town_square")
```

**Terminal Auto-Conversion:**
```
@object edit teleport_stone trigger OnUse teleport("town_square")
```
(Note: " and " are U+201C and U+201D, not U+0022)

**Parser Error:**
```
⚠️ Trigger error: Parse error: Unexpected character: '"'
```

## Root Cause

The trigger DSL tokenizer (`src/tmush/trigger/parser.rs`) expects ASCII straight quotes (U+0022) for string literals. When macOS/terminals substitute smart quotes, the parser encounters unexpected Unicode characters and fails.

### Common Sources of Smart Quotes

1. **macOS text fields** - Auto-convert quotes in system text inputs
2. **Terminal emulators** - Some have "smart quotes" settings enabled
3. **SSH clients** - May apply quote conversion
4. **Copy/paste** - From documents with smart quotes (Word, Pages, etc.)

## Solution

Normalize smart quotes to straight quotes before storing executable trigger scripts. This is done at **two entry points**:

### Entry Point 1: @OBJECT EDIT TRIGGER Command

**File:** `src/tmush/commands.rs` (lines ~8905-8920)

```rust
// Normalize smart/curly quotes to straight quotes for trigger scripts
// macOS and some terminals auto-convert quotes which breaks DSL parsing
let script_code = script_code
    .replace('\u{201C}', "\"")  // " LEFT DOUBLE QUOTATION MARK
    .replace('\u{201D}', "\"")  // " RIGHT DOUBLE QUOTATION MARK
    .replace('\u{2018}', "'")   // ' LEFT SINGLE QUOTATION MARK
    .replace('\u{2019}', "'");  // ' RIGHT SINGLE QUOTATION MARK
```

**Commit:** 766bcdb

### Entry Point 2: Builder Wizard Custom Scripts

**File:** `src/tmush/builder_commands.rs` (lines ~446-456)

```rust
pub fn handle_wizard_step(
    session: &mut WizardSession,
    input: &str,
    store: &TinyMushStore,
) -> Result<String, TinyMushError> {
    let input = input.trim();
    
    // Normalize smart/curly quotes to straight quotes for trigger scripts
    let input = input
        .replace('\u{201C}', "\"")  
        .replace('\u{201D}', "\"")  
        .replace('\u{2018}', "'")   
        .replace('\u{2019}', "'");
```

## Scope Decision

**Systems Requiring Normalization:**
- ✅ **Trigger scripts** (executable code) - CRITICAL
  - @OBJECT EDIT TRIGGER command
  - Builder wizard custom scripts

**Systems NOT Requiring Normalization:**
- ❌ **Display text** (names, descriptions) - Aesthetic preference, no execution
- ❌ **NPC dialog** - Just display text, no parsing
- ❌ **Mail/messages** - No script execution
- ❌ **Quest descriptions** - Display only
- ❌ **Recipe descriptions** - Display only

**Rationale:** Only executable trigger scripts are parsed by the DSL parser. All other text is display-only and can safely contain smart quotes as an aesthetic choice.

## Testing

### Test Files Created

1. **tests/trigger_quote_types.rs**
   - `test_teleport_simple()` - Verifies straight quotes work
   - `test_smart_quotes()` - Confirms smart quotes are rejected with error
   - `test_char_codes()` - Documents Unicode codepoints

2. **tests/trigger_smart_quote_normalization.rs**
   - `test_trigger_smart_quotes_normalized()` - Full integration test for @OBJECT EDIT
   - Verifies normalization converts smart quotes to straight quotes
   - Confirms normalized script parses successfully

3. **tests/wizard_smart_quote_normalization.rs**
   - `test_wizard_normalizes_smart_quotes_in_custom_script()` - Builder wizard custom script
   - `test_wizard_normalizes_smart_quotes_in_message()` - Builder wizard message template
   - Tests both wizard input paths

4. **tests/trigger_unicode.rs**
   - `test_unicode_emoji_in_string()` - Emoji support in strings
   - `test_unicode_in_compound_expression()` - Complex expressions
   - Ensures fix doesn't break legitimate Unicode usage

### Test Coverage

- ✅ Direct command input (@OBJECT EDIT TRIGGER)
- ✅ Builder wizard custom scripts
- ✅ Builder wizard message templates
- ✅ Emoji and Unicode support preserved
- ✅ Parser error messages for invalid syntax

## Verification Steps

### For Developers

1. **Run full test suite:**
   ```bash
   cargo test
   ```
   Expected: All ~640+ tests pass

2. **Test specific normalization:**
   ```bash
   cargo test --test wizard_smart_quote_normalization
   cargo test --test trigger_smart_quote_normalization
   ```

### For End Users

1. **Rebuild the binary:**
   ```bash
   cargo build --release
   ```

2. **Test @OBJECT EDIT TRIGGER command:**
   ```
   @object create teleport_stone
   @object edit teleport_stone trigger OnUse teleport("town_square")
   ```
   Expected: Trigger saved successfully (even if terminal converts quotes)

3. **Test builder wizard:**
   ```
   /create
   [Select an object]
   [Select trigger type: 2 (OnUse)]
   [Select action: 5 (Custom script)]
   teleport("town_square")
   ```
   Expected: Wizard completes successfully

4. **Verify trigger execution:**
   ```
   use teleport_stone
   ```
   Expected: Player teleported to town_square

## Related Commits

- **766bcdb** - Initial fix for @OBJECT EDIT TRIGGER command
- **[pending]** - Extended fix to builder wizard custom scripts

## Additional Notes

### Why Not Fix the Parser?

We chose normalization over parser modification because:

1. **Principle of Least Surprise**: Straight quotes are the standard for string literals in programming languages
2. **Consistency**: All stored scripts use ASCII quotes
3. **Debugging**: Error messages show expected characters clearly
4. **Copy/Paste**: Code examples from documentation work correctly
5. **Terminal Independence**: Solution works regardless of terminal settings

### Future Considerations

If other DSL parsing issues arise from terminal character conversion, consider:

1. **Pre-validation**: Warn users when input contains unexpected Unicode
2. **Documentation**: Add terminal configuration guide for developers
3. **Editor Integration**: Recommend text editors over terminal input for complex scripts

## References

- Unicode Standard: https://www.unicode.org/charts/PDF/U2000.pdf
- macOS Smart Quotes: System Preferences → Keyboard → Text → "Use smart quotes and dashes"
- Terminal.app Quote Conversion: Preferences → Profiles → Advanced → Character Encoding
