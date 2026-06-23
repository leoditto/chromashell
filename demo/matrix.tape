Output demo/matrix.gif

Set Shell zsh
Set FontSize 16
Set Width 1200
Set Height 700
Set Framerate 30
Set TypingSpeed 80ms

Hide
Type `export PATH="$PWD/target/release:$PATH"`
Enter
Sleep 500ms
Show

Type "cs matrix"
Enter
Sleep 1s

Type "echo 'Hello from chromashell!'"
Enter
Sleep 800ms

Type "ls -la src/"
Enter
Sleep 800ms

Type "cat Cargo.toml | head -5"
Enter
Sleep 1s

Hide
Type `exit`
Enter
Sleep 300ms
# Ctrl+\ to quit chromashell
Ctrl+\
Sleep 500ms
