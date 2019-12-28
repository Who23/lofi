install : 
	@echo "Building project...."
	@cargo build --release 2>/dev/null
	@echo "Creating .config/lofi directories..."
	@if ! [ -d ~/.config ]; then\
		mkdir ~/.config;\
	fi
	@if ! [ -d ~/.config/lofi ]; then\
		mkdir ~/.config/lofi;\
	fi
	@if ! [ -d ~/.config/lofi/data ]; then\
		mkdir ~/.config/lofi/data;\
	fi
	@if ! [ -d ~/.config/lofi/data/music ]; then\
		mkdir ~/.config/lofi/data/music;\
	fi
	@if ! [ -f ~/.config/lofi/config ]; then\
		touch ~/.config/lofi/config;\
	fi
	@if ! [ -f ~/.config/lofi/data/last_playlist ]; then\
		touch ~/.config/lofi/data/last_playlist;\
	fi
	@echo "Finished! The compiled binary is located at ./target/release/lofi"
	@echo "If you're installing this through homebrew, this binary should be installed automatically"
	@echo "Otherwise, you'll have to put the binary in your path"

