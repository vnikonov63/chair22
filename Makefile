.SECONDARY:

tests/%.s: tests/%.snek src/main.rs
	cargo run --target x86_64-apple-darwin -- $< tests/$*.s

tests/%.run: tests/%.s runtime/start.rs
	nasm -f macho64 tests/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc --target x86_64-apple-darwin -L runtime/ runtime/start.rs -o tests/$*.run

clean:
	cargo clean
	rm -f tests/*.a tests/*.s tests/*.run tests/*.of

repl:
	cargo run -- -i