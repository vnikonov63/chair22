.SECONDARY:

#TODO: This are the commands that we only use in Ahead of Time compiling, so it would be enough
# to jsut have the -c flag here, right?

tests/%.s: tests/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- -c $< tests/$*.s

tests/%.run: tests/%.s runtime/start.rs
	nasm -f macho64 tests/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc --target x86_64-apple-darwin -L runtime/ runtime/start.rs -o tests/$*.run