.SECONDARY:

UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
NASM_FMT := macho64
RUSTC_TARGET_FLAG := --target x86_64-apple-darwin
CARGO_TARGET_FLAG := --target x86_64-apple-darwin
else
NASM_FMT := elf64
RUSTC_TARGET_FLAG :=
CARGO_TARGET_FLAG :=
endif

tests/%.s: tests/%.snek src/main.rs
	cargo run $(CARGO_TARGET_FLAG) -- $< tests/$*.s

tests/%.run: tests/%.s runtime/start.rs
	nasm -f $(NASM_FMT) tests/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc $(RUSTC_TARGET_FLAG) -L runtime/ runtime/start.rs -o tests/$*.run

clean:
	cargo clean
	rm -f tests/*.a tests/*.s tests/*.run tests/*.of

repl:
	cargo run -- -i