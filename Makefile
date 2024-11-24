# -*- Makefile -*-

## TODO: Add stuff here 

bin/:
	mkdir -p $@

.PHONY: clean	
clean:
	-$(RM) -r bin