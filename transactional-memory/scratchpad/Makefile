files := $(wildcard *.rs)

log := RUSTC_LOG=rustc_mir=debug

# RUSTC_LOG=rustc_mir=debug rustc +stage1 main.rs --emit mir &> main.log
compile =            $(log) rustc +stage1 $(file) --emit mir > $(patsubst %.rs,%.log,$(file))

*.log: compile

*.mir: compile

# Piping output: https://stackoverflow.com/a/19557508
compile: *.rs
	-@$(foreach file, $(files), $(compile) 2>&1;)
	@echo "done 🦀"

clean:
	-@rm -f *.mir;
	-@rm -f *.log;
	@echo "clean"

