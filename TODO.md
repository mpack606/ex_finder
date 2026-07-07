*High prio:
[] Tabs
[] Add keyboard shortcuts
    - ctrl f - search
    - ctrl c - copy
    - ctrl v - paste
    - ctrl x - cut
    - delete - delete
[] Right click menu
    - open with...
    - cut
    - rename
    - unzip file
    - zip selected
    - create new folder
    - create new file

*Medium prio:
[] Add sorting based on date, name, size, or type.
[] List view with item details
[] Navigation bar with ability to click path to navigate there
[] Add box selection with mouse

*Low prio:
[] Recent locations (display last 5 folders opened below quick access)
[] Get file info (opens new small window with size, extension, full name)
[] Swap places of navigation bar and bottom bar
[] Improve text display (mostly font)

Technical improvements:
[] Make IO async so big folders dont wait for eternity to open
[] Separate system operations to separate service instead of intervening with grid_view
[] Display errors in bottom_bar
[] Create file with layout constants