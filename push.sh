 cargo build --target=x86_64-pc-windows-gnu --release && 
 cp target/x86_64-pc-windows-gnu/release/walker.exe download/windows/walker.exe &&
 echo "Saved to download/windows" &&
 cargo build -r && 
 cp target/release/walker download/macOS/walker &&
 echo "Saved to download/macOS" &&
 gth --input "README.md" --output "index.html" -w -b && 
 commit -m $1