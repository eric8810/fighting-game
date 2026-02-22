# Manual Black-Box Test Checklist for Text Rendering Integration

## Test Environment
- **Build**: `cargo run --release`
- **Expected Window**: 800x600 pixels, "Tickle Fighting Engine" title
- **Controls**:
  - P1: WASD (move), Space (attack), Escape (pause)
  - P2: Arrow keys (move), Enter (attack)

---

## Test Suite 1: Main Menu Text Rendering

### Test 1.1: Main Menu Display
**Steps**:
1. Launch the game
2. Observe the main menu

**Expected Results**:
- ✅ Title text "TICKLE FIGHTING ENGINE" visible at top center (large, dark text)
- ✅ Three menu options visible:
  - "VS MODE" (highlighted in white if selected)
  - "TRAINING" (gray if not selected)
  - "QUIT" (gray if not selected)
- ✅ Text is crisp and readable (not blurry or pixelated)
- ✅ No colored rectangles where text should be

**Pass/Fail**: ___________

---

### Test 1.2: Menu Navigation
**Steps**:
1. From main menu, press W (up) and S (down) repeatedly
2. Observe menu selection changes

**Expected Results**:
- ✅ Selected menu item turns white
- ✅ Unselected items turn gray
- ✅ Cursor wraps around (from QUIT to VS MODE and vice versa)
- ✅ Text color changes are smooth and immediate

**Pass/Fail**: ___________

---

## Test Suite 2: In-Game HUD Text Rendering

### Test 2.1: Player Labels
**Steps**:
1. Select "VS MODE" from main menu
2. Observe the top of the screen

**Expected Results**:
- ✅ "P1" label visible above left health bar (green color)
- ✅ "P2" label visible above right health bar (red color)
- ✅ Labels are properly aligned with health bars
- ✅ Text is legible at small size

**Pass/Fail**: ___________

---

### Test 2.2: Round Timer Display
**Steps**:
1. Start a VS MODE match
2. Watch the timer at top center

**Expected Results**:
- ✅ Timer shows "99" at match start (yellow color)
- ✅ Timer counts down: 99 → 98 → 97... (one per second)
- ✅ Timer turns RED when it reaches 10 or below
- ✅ Timer shows "00" when time expires
- ✅ Digits are clearly readable (not seven-segment blocks)

**Pass/Fail**: ___________

---

### Test 2.3: Combo Counter
**Steps**:
1. In VS MODE, position P1 near P2
2. Press Space repeatedly to attack P2
3. Land 2+ consecutive hits quickly

**Expected Results**:
- ✅ After 2nd hit, text appears: "2 HITS" (gold color)
- ✅ After 3rd hit: "3 HITS"
- ✅ After 4th hit: "4 HITS"
- ✅ Combo counter disappears after ~1 second of no hits
- ✅ Text is positioned near the attacking player
- ✅ Both P1 and P2 can trigger combo counters independently

**Pass/Fail**: ___________

---

## Test Suite 3: Pause Menu Text

### Test 3.1: Pause Menu Display
**Steps**:
1. During a match, press Escape
2. Observe the pause overlay

**Expected Results**:
- ✅ "RESUME" option visible (white if selected)
- ✅ "QUIT TO MENU" option visible (gray if not selected)
- ✅ Text is centered in the pause overlay box
- ✅ Background is semi-transparent, game visible behind

**Pass/Fail**: ___________

---

### Test 3.2: Pause Menu Navigation
**Steps**:
1. In pause menu, press W/S to navigate
2. Press Space to select

**Expected Results**:
- ✅ Selection highlight changes between options
- ✅ "RESUME" returns to game
- ✅ "QUIT TO MENU" returns to main menu
- ✅ Text remains readable during navigation

**Pass/Fail**: ___________

---

## Test Suite 4: Round/Match End Text

### Test 4.1: Round End Banner
**Steps**:
1. Start VS MODE
2. Deplete P2's health to zero (P1 attacks repeatedly)
3. Observe the round end screen

**Expected Results**:
- ✅ Large banner text appears: "PLAYER 1 WINS!" (white, centered)
- ✅ Text is displayed for ~2 seconds
- ✅ Next round starts automatically
- ✅ Banner is clearly visible over the game

**Pass/Fail**: ___________

---

### Test 4.2: Match End Banner
**Steps**:
1. Win 2 rounds as P1 (best of 3)
2. Observe the match end screen

**Expected Results**:
- ✅ Banner text: "PLAYER 1 WINS THE MATCH!" (white, centered)
- ✅ Text is slightly smaller than round end banner
- ✅ Returns to main menu after display

**Pass/Fail**: ___________

---

### Test 4.3: Draw Scenario
**Steps**:
1. Start VS MODE
2. Let timer run to 00 with both players at equal health
3. Observe the round end

**Expected Results**:
- ✅ Banner text: "DRAW" (white, centered)
- ✅ Round is replayed (no score change)

**Pass/Fail**: ___________

---

## Test Suite 5: Training Mode

### Test 5.1: Training Mode Text
**Steps**:
1. Select "TRAINING" from main menu
2. Observe the HUD

**Expected Results**:
- ✅ All HUD text renders correctly (P1/P2 labels, timer)
- ✅ Timer still counts down normally
- ✅ Health bars refill when depleted (infinite HP)
- ✅ Combo counter works normally

**Pass/Fail**: ___________

---

## Test Suite 6: Stress Testing

### Test 6.1: Rapid Menu Navigation
**Steps**:
1. Rapidly press W/S in main menu for 10 seconds
2. Observe text rendering

**Expected Results**:
- ✅ No text flickering or corruption
- ✅ Selection changes remain smooth
- ✅ No crashes or freezes

**Pass/Fail**: ___________

---

### Test 6.2: Long Match Duration
**Steps**:
1. Start VS MODE
2. Let the match run for full 99 seconds without attacking
3. Observe timer and text throughout

**Expected Results**:
- ✅ Timer counts down smoothly for entire duration
- ✅ No text rendering glitches over time
- ✅ Color change at 10 seconds works correctly
- ✅ "00" displays correctly at timeout

**Pass/Fail**: ___________

---

### Test 6.3: Multiple Combo Counters
**Steps**:
1. In VS MODE, trigger P1 combo (2+ hits)
2. Immediately switch and trigger P2 combo
3. Observe both combo counters

**Expected Results**:
- ✅ Both combo counters can display simultaneously
- ✅ Each counter tracks independently
- ✅ No text overlap or corruption
- ✅ Counters timeout independently

**Pass/Fail**: ___________

---

## Test Suite 7: Visual Quality

### Test 7.1: Text Clarity
**Steps**:
1. Observe all text elements in various game states
2. Check for visual artifacts

**Expected Results**:
- ✅ All text is anti-aliased and smooth
- ✅ No jagged edges on letters
- ✅ Text is not blurry or pixelated
- ✅ Consistent font rendering across all UI elements

**Pass/Fail**: ___________

---

### Test 7.2: Color Accuracy
**Steps**:
1. Verify text colors match specifications:
   - P1 label: Green
   - P2 label: Red
   - Timer (normal): Yellow
   - Timer (low): Red
   - Combo counter: Gold
   - Menu selected: White
   - Menu unselected: Gray

**Expected Results**:
- ✅ All colors match specifications
- ✅ Colors are vibrant and distinguishable
- ✅ No color bleeding or artifacts

**Pass/Fail**: ___________

---

## Test Suite 8: Regression Testing

### Test 8.1: Quad Rendering Still Works
**Steps**:
1. Observe all colored rectangles in the game:
   - Health bars (green/red)
   - Power gauge stocks (blue)
   - Fighter rectangles
   - Menu backgrounds

**Expected Results**:
- ✅ All colored quads render correctly
- ✅ No visual corruption from text rendering
- ✅ Quads and text layer correctly (text on top)

**Pass/Fail**: ___________

---

### Test 8.2: Performance
**Steps**:
1. Play a full match with active combat
2. Observe frame rate and responsiveness

**Expected Results**:
- ✅ Game runs at smooth 60 FPS
- ✅ No stuttering or frame drops
- ✅ Input response is immediate
- ✅ Text rendering doesn't impact performance

**Pass/Fail**: ___________

---

## Summary

**Total Tests**: 23
**Passed**: _____ / 23
**Failed**: _____ / 23

**Critical Issues Found**:
1. ___________________________________________
2. ___________________________________________
3. ___________________________________________

**Minor Issues Found**:
1. ___________________________________________
2. ___________________________________________
3. ___________________________________________

**Tester Name**: ___________
**Date**: ___________
**Build Version**: ___________
**Platform**: Windows 11 / Other: ___________

---

## Notes

- If any test fails, take a screenshot and note the exact steps to reproduce
- Check the console output for any error messages
- Report any crashes with the error log
- Note any unexpected behavior even if tests pass
