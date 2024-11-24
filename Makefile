# -*- Makefile -*-

## TODO: Add stuff here 

all: bin/sort_files
default: all

bin/sort_files: sort_files.rs | bin/
	rustc $< -o $@

bin/:
	mkdir -p $@

.PHONY: clean	
clean:
	-$(RM) -r bin