# TODO

---

## Operations
### Draw
    - [x] point
    - [x] line
    - [ ] path
    - [ ] rect
    - [ ] circle
    - [ ] text
    - [ ] picture

### Adjust
    - [ ] move
    - [ ] scale
    - [ ] rotate
    - [ ] duplicate

### Flow
    - [ ] loop

---

# Data
    - [ ] Data pane for pictures
        - [ ] Number data
        - [ ] Point data
        - [ ] Color data
    - [ ] Edit data for primative commands
        - Stroke/fill color, line thickness, font size, etc.

---

## General Concepts
    - [ ] Cursor (active step)
        - [ ] Visually see which steps are active (up to the current, the rest should be greyed/muted)
        - [ ] Canvas should reflect only up to the active
        - [ ] Use arrow keys to navigate
    - [ ] Zoom/pan
    - [x] Snap points
        - [x] as destinations
        - [ ] add a snap point at all intersections between drawables?
    - [x] Recognize objects that have been drawn (like circles, paths, rects, lines, points, etc.)
    - [-] Point/click selection of objects
    - [ ] Expressions (?) as in, draw point at max(some_array) / len(some_array)
        - Probably use Rhai for this
    - [ ] Guides (temporary variables, hidden)
