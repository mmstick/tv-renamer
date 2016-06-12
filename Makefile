DESTDIR = /usr

all: gtk cli

gtk:
	cargo build --release --features "enable_gtk"
	mv "target/release/tv-renamer" "target/release/tv-renamer-gtk"

cli:
	cargo build --release

install: install-cli install-gtk

install-cli:
	install -Dm 755 "target/release/tv-renamer" "${DESTDIR}/bin/"
	install -Dm 644 README.md "${DESTDIR}/share/doc/tv-renamer/README"
	install -Dm 644 LICENSE "${DESTDIR}/share/licenses/tv-renamer/COPYING"

install-gtk:
	install -Dm 755 "target/release/tv-renamer-gtk" "${DESTDIR}/bin/"
	install -Dm 644 "assets/tv-renamer.desktop" "${DESTDIR}/share/applications/"
	install -Dm 644 README.md "${DESTDIR}/share/doc/tv-renamer/README"
	install -Dm 644 LICENSE "${DESTDIR}/share/licenses/tv-renamer/COPYING"

create-tar:
	install -Dm 755 "target/release/tv-renamer-gtk" "tv-renamer/bin/"
	install -Dm 755 "target/release/tv-renamer" "tv-renamer/bin/"
	install -Dm 644 "assets/tv-renamer.desktop" "tv-renamer/share/applications/"
	tar cf - "tv-renamer" | xz -zf > "tv-renamer-binaries.tar.xz"

uninstall:
	rm "${DESTDIR}/tv-renamer"
	rm "${DESTDIR}/tv-renamer-gtk"

clean:
	cargo clean
