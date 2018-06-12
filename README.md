# Serwer DHCP

## Opis projektu

Serwer DHCP napisany w języku Rust i w modelu aktorów przy użyciu biblioteki Actix.
Obsługuje dynamiczne nadawanie klientom adresów IP z określonej puli, niezależnie od długości maski, na określony czas.
Umożliwia konfigurację statycznie przydzielanych adresów.
Przydziela maskę sieciową, adres routera i serwera DHCP, DNSy. 

## Zawartość plików źródłowych

Pliki źródłowe obecne są w katalogu `src`. Główny moduł `main.rs` zawiera funkcję `main()`, która uruchamia tworzy socket, uruchamia wątek obsługujący odbierający pakiety z socketa i system aktorów,
Moduł `config.rs` zawiera strukturę opisującą konfigurację serwera DHCP: pulę adresów, maskę, adres serwera, czas dzierżawy etc.
Moduł `dhcp_frames.rs` zawiera strukturę pakietu DHCP i funkcje jego (de)serializacji z/do ciągu bajtów.
Moduł `io_actor.rs` zawiera aktora odbierającego pakiety DHCP wygenerowane przez serwer i wysyłającego je na socket.
Moduł `server_actor.rs` zawiera aktora obsługującego logikę serwera DHCP.

## Kompilacja i uruchamianie

Do kompilacji programu wymagany jest kompilator, biblioteka standardowa (stable + nightly) i narzędzia Rusta.
Można je pobrać ze strony https://rustup.rs/. Zależności programu pobierze za nas przy kompilacji program `cargo`.

Aby skompilować program użyjemy narzędzia `cargo`. W głównym katalogu projektu wydajemy polecenie:
```bash
$ cargo build
```

Ze względu na użycie portu 67 przez nasz socket, program potrzebuje praw roota. Nie powiniśmy wykonywać polecenia `cargo run` jako root, dlatego najlepiej uruchomić program bezpośrednio z katalogu projektu.
```bash
# ./target/debug/rust_dhcp
```