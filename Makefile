DESTDIR = /usr
version = $(shell awk 'NR == 3 {print substr($$3, 2, length($$3)-2)}' Cargo.toml)

all:
	cargo build --release

install:
	install -Dm 755 "target/release/tv-renamer" "${DESTDIR}/bin/tv-renamer"
	ln -sf "${DESTDIR}/bin/tv-renamer" "${DESTDIR}/bin/tv-renamer-gtk"
	install -Dm 644 "assets/tv-renamer.desktop" "${DESTDIR}/share/applications/tv-renamer.desktop"
	install -Dm 644 README.md "${DESTDIR}/share/doc/tv-renamer/README"
	install -Dm 644 LICENSE "${DESTDIR}/share/licenses/tv-renamer/COPYING"

tar:
	install -Dm 755 "target/release/tv-renamer" "tv-renamer/bin/tv-renamer"
	ln -sf "tv-renamer/bin/tv-renamer" "tv-renamer/bin/tv-renamer-gtk"
	install -Dm 644 "assets/tv-renamer.desktop" "tv-renamer/share/applications/tv-renamer.desktop"
	install -Dm 644 README.md "tv-renamer/share/doc/tv-renamer/README"
	install -Dm 644 LICENSE "tv-renamer/share/licenses/tv-renamer/COPYING"
	tar cf - "tv-renamer" | xz -zf > "tv-renamer_$(version)_$(shell uname -m).tar.xz"

deb:
	dpkg -s libgtk-3-dev >/dev/null 2>&1 || sudo apt install libgtk-3-dev -y
	dpkg -s libssl-dev >/dev/null 2>&1 || sudo apt install libssl-dev -y
	cargo build --release
	strip --strip-unneeded target/release/tv-renamer
	cargo deb --no-build


uninstall:
	rm "${DESTDIR}/bin/tv-renamer"
	rm "${DESTDIR}/bin/tv-renamer-gtk"
	rm "${DESTDIR}/share/applications/tv-renamer.desktop"
	rm "${DESTDIR}/share/doc/tv-renamer/README"
	rm "${DESTDIR}/share/licenses/tv-renamer/COPYING"

clean:
	cargo clean
	rm target/debian -R
