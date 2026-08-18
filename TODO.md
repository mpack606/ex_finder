*High prio:
[] Add keyboard shortcuts
    - ctrl f - search
    - ctrl c - copy
    - ctrl v - paste
    - ctrl x - cut
    - delete - delete
    - enter - rename
    - ctrl n - new file
    - ctrl d - new directory?
[] Right click menu
    - open with...
    - cut
    - zip selected
    - create new file

*Medium prio:
[] Add sorting based on date, name, size, or type.
[] List view with item details
[] Improve selection:
    - drag to box select
    - CMD + click to select multiple files
    - SHIFT + click to select everything inbetween
[] Drag and drop
[] Search options - look in folders
[] Navigation bar with ability to click path to navigate there

*Low prio:
[] Add some interactivity - for example rezipping same file doesn't indicate any change (maybe log events)
[] Display tabs side by side
[] Recent locations (display last 5 folders opened below quick access)
[] Get file info (opens new small window with size, extension, full name)
[] Swap places of navigation bar and bottom bar
[] Improve text display (mostly font)
[] Actions should focus on folder/file after action

Technical improvements:
[] Make IO async so big folders dont wait for eternity to open
[] Separate system operations to separate service instead of intervening with grid_view
[] Display errors in bottom_bar
[] Create file with layout constants

Bugs:
[] When scrolled to end right click moves slidebar to top