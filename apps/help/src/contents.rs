//! The knowledge base of the help app — categories, articles, texts.
//!
//! Content for both audiences: users (usage, colors, permissions)
//! and developers (architecture, rules, app building). New articles:
//! just add them here — the app renders sidebar, cards and reading view from it.

pub struct Article {
    pub title: &'static str,
    pub teaser: &'static str,
    pub content: &'static str,
}

pub struct Category {
    pub name: &'static str,
    pub article: &'static [Article],
}

pub const CATEGORIES: &[Category] = &[
    Category {
        name: "Erste Schritte",
        article: &[
            Article {
                title: "Was ist MatrixKit?",
                teaser: "Das UI-Fundament von Matrix — eine Sprache, ein Look, ein Gefühl.",
                content: "MatrixKit ist das App-Fundament des Matrix-Systems: eine Sammlung von Bausteinen, mit denen alle eigenen Apps gebaut werden — vollständig in Rust.\n\nDie Idee dahinter: Auf einem Referenzsystem fühlt sich jede gute App wie ein Teil des Ganzen an, weil alle dieselbe Basis nutzen (Leitbild-UI). MatrixKit übernimmt dieses Prinzip: Fenster, Kopfzeile, Schalter, Farben, Bewegung und PermissionSet kommen aus dem Rahmen — die App selbst ist nur noch Inhalt.\n\nDeshalb sehen Matrix-Apps nicht nur gleich aus, sie verhalten sich auch gleich: gleiche Gesten, gleiche Abstände, gleiche Reaktionen. Und weil die Farben aus derselben Quelle stammen wie die des restlichen Systems, ist eine MatrixKit-App von der Shell visuell nicht zu unterscheiden.",
            },
            Article {
                title: "Die Matrix-Apps",
                teaser: "Monitor, Farben, Hilfe — und wie sie in Launcher und Dock leben.",
                content: "Die Familie ist inzwischen auf über dreißig Apps gewachsen — vom Dateimanager über Terminal, Markdown und Player bis zu Dock, Bar und Greeter. Hier die Gründungsmitglieder als Rundgang (alle weiteren findest du im Launcher):\n\nMatrix Einstellungen — die Systemeinstellungen im Matrix-Stil: Erscheinungsbild (Icon-Stil, Bewegung), Anzeige und mehr. Jede Änderung wirkt sofort und landet als einfache Datei unter ~/.config/matrix/.\n\nMatrix Monitor — Prozessor, Arbeitsspeicher, Datenträger und Netzwerk auf einen Blick, mit federnden Balken und einer Verlaufskurve der letzten zwei Minuten.\n\nMatrix Farben — die lebende System-Palette. Jede Farbrolle mit Hex-Wert; ein Klick kopiert ihn in die Zwischenablage.\n\nMatrix Klänge — die Systemklänge: anhören, einzeln oder komplett abschalten. Die Klänge selbst sind Code: von Grund auf in Rust synthetisiert, alle aus einer Familie tiefer, ruhiger Pulse.\n\nMatrix Codes — ein 2FA-Authenticator: zeigt deine Zwei-Faktor-Codes (TOTP) mit ablaufendem Ring, ein Klick kopiert. Die Codes werden von Grund auf in Rust berechnet (mit den RFC-6238-Testvektoren belegt); Konten liegen hinter dem Passwort-Schloss.\n\nMatrix Schlüssel — richtet einen USB-Stick als Login-Schlüssel ein: danach meldest du dich mit dem Stick ODER dem Passwort an, beide gleichwertig. Das Passwort bleibt immer gültig, du kannst dich nicht aussperren.\n\nMatrix Hilfe — diese App. Mit Volltextsuche oben rechts: tippen zeigt Treffer aus allen Kapiteln, Esc räumt die Suche wieder weg.\n\nUnd eine achte, besondere: Matrix Wiederherstellung — die geführte Recovery des Wächters. Sie wohnt nicht im Launcher, sondern erscheint am Login-Bildschirm, wenn jemand sein Passwort vergessen hat (siehe Kapitel „Der Wächter“).\n\nAlle Apps findest du im Launcher (kurz die Super-Taste drücken und den Namen tippen). Laufende Apps erscheinen im Dock — mit ihrem lebenden Icon, das die aktuellen Systemfarben trägt.",
            },
            Article {
                title: "Lebende Farben",
                teaser: "Warum jede App sofort mitfärbt, wenn das Hintergrundbild wechselt.",
                content: "Matrix hat eine einzige Farbquelle: dein Hintergrundbild. Daraus errechnet das System eine vollständige Material-Palette — und diese eine Wahrheit fließt überall hin: Shell, Fenster, Terminal, Firefox, Login-Bildschirm und eben auch in jede MatrixKit-App.\n\nMatrixKit-Apps beobachten die Palette live: Wechselst du das Wallpaper, färben laufende Apps binnen zwei Sekunden um — ohne Neustart. Sogar die App-Icons in Launcher und Dock werden neu gezeichnet, denn sie sind keine statischen Bilder, sondern werden aus der aktuellen Palette generiert.\n\nDazu kommt der Sonnen-Rhythmus: Bei Sonnenaufgang wird das System hell, bei Sonnenuntergang dunkel — täglich neu berechnet für deinen Standort.",
            },
        ],
    },
    Category {
        name: "Bedienung",
        article: &[
            Article {
                title: "Fenster & Gesten",
                teaser: "Ziehen, Größe ändern, Schließen — ohne klassische Fensterdeko.",
                content: "MatrixKit-Fenster haben eine eigene Kopfzeile im System-Look statt einer Fremd-Dekoration.\n\nVerschieben: die Kopfzeile mit der Maus greifen und ziehen — das ist dieselbe native Compositor-Geste wie bei den System-Fenstern.\n\nGröße ändern: an der rechten Kante, der Unterkante oder der Ecke unten rechts ziehen. Jede App hat eine sinnvolle Mindestgröße; kleiner geht es bewusst nicht.\n\nSchließen: der ×-Knopf rechts in der Kopfzeile.\n\nAlles Interaktive in der Kopfzeile zeigt sich beim Überfahren mit der Maus (Hover-Aufhellung) — was keinen Hover-Effekt hat, ist auch nicht klickbar.

Tastatur: Tab wandert durch die Elemente (ein farbiger Ring zeigt den Fokus), Enter aktiviert, Esc schließt Ebenen oder führt zurück. Shift+Tab wandert rückwärts.",
            },
            Article {
                title: "Die Root-Ebene",
                teaser: "Ein Klick auf den App-Namen — Über, Einstellungen, PermissionSet.",
                content: "Jede MatrixKit-App hat eine Verwaltungsebene, die Root-Ebene. Du erreichst sie mit einem Klick auf den App-Namen in der Kopfzeile.\n\nDie App dimmt dabei aus — wie eine angehaltene virtuelle Machine — und darüber erscheint ein Panel mit zwei Dingen: „Über“ (Name, Version, Beschreibung, wie man es vom Leitbild kennt) und den PermissionSet der App.\n\nSchließen: auf „Fertig“ klicken, auf den abgedunkelten Bereich — oder einfach Esc drücken. Die App läuft dabei ungestört weiter — pausiert wird nur die Optik.

Die Ebene gleitet federnd herein. Gefährliche Aktionen (etwa das Löschen eines Sticks) fragen grundsätzlich noch einmal nach — Abbrechen ist immer der nächstliegende Weg. Wer weniger Bewegung möchte: eine Datei ~/.config/matrix/bewegung mit dem Inhalt „reduziert“ schaltet alle Übergänge auf sofortige Zustände um.",
            },
            Article {
                title: "PermissionSet",
                teaser: "Bindend, nicht symbolisch: Ohne Recht kein Zugriff — wortwörtlich.",
                content: "Die Berechtigungen in der Root-Ebene sind keine Deko. Der MatrixKit-Rahmen prüft sie, BEVOR die App auf etwas zugreift: Ist ein Schalter aus, wird der Zugriff gar nicht erst ausgeführt.\n\nÄnderungen sind durch dein Konto-Passwort geschützt (das Schloss, wie im Leitbild): Solange die Sektion verriegelt ist, sind die Schalter stumpf — erst die Passwort-Bestätigung entsperrt sie, und beim Schließen der Root-Ebene verriegelt sie wieder.\n\nBeispiel: Schaltest du beim Systemmonitor „Netzwerk“ aus, fragt die App die Netzwerk-Schnittstellen nicht mehr ab — die Zeile zeigt „Berechtigung aus“. Schaltest du bei Matrix Farben die Zwischenablage aus, kopiert der Klick nichts und die Fußzeile erklärt warum.\n\nIm Katalog stehen außerdem Kamera, Mikrofon und Datei-Zugriffe bereit: Sobald eine künftige App sie nutzt, laufen sie durch dieselbe Prüfstelle und erscheinen automatisch in ihrer Root-Ebene.\n\nDeine Einstellungen überleben App-Neustarts — gespeichert wird pro App unter ~/.config/matrix/permissions/.",
            },
            Article {
                title: "Die unendliche Leinwand",
                teaser: "Matrix' eigene Arbeitsfläche: Fenster nebeneinander, nie minimiert.",
                content: "Die Leinwand ist Matrix' eigene Arbeitsfläche (Einstellungen → Allgemein → Leinwand): Fenster überlappen nicht — sie leben nebeneinander auf einer Fläche, die in alle Richtungen wächst.\n\nNavigieren heute: Super + Mausrad wandert durch die Spalten (links/rechts), Super + Bild-Tasten durch die Arbeitsflächen (oben/unten) — zusammen ist das ein Raster in zwei Richtungen. Die Übersicht (Super + O) zoomt heraus und zeigt die ganze Fläche als Karte: Fenster greifen, verschieben, hinspringen — wie die App-Wolke einer das Leitbild Watch.\n\nMinimieren gibt es auf der Leinwand nicht mehr. Die \u{2212}-Ampel legt stattdessen den PRIVATSCHLEIER über das Fenster: Es bleibt an seinem Ort und in seiner Größe, nur der Inhalt ist verdeckt — ein Klick auf die Fläche öffnet es wieder. So verliert kein Fenster je seinen Platz, und neugierige Blicke sehen trotzdem nichts.\n\nAusblick: Das freie Ziehen der Sicht mit gedrückter Maustaste auf dem Leerraum — in jede Richtung, wie auf einer echten Leinwand — braucht einen Eingriff in den Compositor und steht als nächster großer Schritt auf der Roadmap.",
            },
            Article {
                title: "Matrix Einstellungen",
                teaser: "Die Einstellungs-Zentrale: Regler, Schalter, Stufen — alles wirkt sofort.",
                content: "Matrix Einstellungen ist die Einstellungs-Zentrale des Systems — du findest sie an erster Stelle im Dock oder tippst „Einstellungen“ im Launcher.\n\nErscheinungsbild: Der Icon-Stil schaltet zwischen farbigen und auf die Palette getönten App-Symbolen — die Icons in Dock und Launcher werden sofort neu gezeichnet. „Bewegung reduzieren“ beschränkt Federn und Übergänge auf das Nötigste.\n\nAnzeige: Anzeigegröße (Schieberegler mit Prozentwert) und Eckenradius (±-Stufen) sind Vormerkungen für den Shell-Ausbau — sie werden gespeichert und künftig systemweit gelesen.\n\nDie Bedienelemente folgen einer festen Grammatik: Ein Schieberegler wächst unter der Maus, an einer Grenze dimmt die ±-Taste und reagiert nicht, die aktive Auswahl trägt die Akzentfarbe. Diese Grammatik gilt in allen Matrix-Apps gleich.\n\nSoftwareupdate (R89): Das ganze Betriebssystem wird hier geprüft und aktualisiert — wie beim Leitbild als Einstellungs-Bereich, nicht als eigene App. „Matrix ist auf dem neuesten Stand“ mit Häkchen, oder eine Update-Karte mit „Was ist neu“ und einem Knopf; der Neustart übernimmt es. Zurückrollen geht jederzeit im Startmenü.\n\nDas Prinzip dahinter: Jede Einstellung ist eine einfache, lesbare Datei unter ~/.config/matrix/ — die App ist nur die Oberfläche darüber. Nichts ist versteckt, alles lässt sich auch von Hand ändern oder sichern.",
            },
            Article {
                title: "Matrix Dateien",
                teaser: "Der Dateimanager: Galerie, Alben, Band — und Auswahl wie im Leitbild.",
                content: "Matrix Dateien ist der Dateimanager des Systems: links die Sidebar mit Favoriten und Orten, rechts die Liste — oder die Galerie, sobald du in Bilder- und Video-Ordnern unterwegs bist.\n\nAuswahl (R88): Ein Klick wählt einen Eintrag. Strg+Klick nimmt weitere dazu oder wieder heraus, Shift+Klick spannt den Bereich vom zuletzt Gewählten bis hierhin, Strg+A wählt alles Sichtbare. Die Fußzeile zählt mit („3 von 24 ausgewählt“). Entf legt die ganze Auswahl in den Papierkorb; auch Kontextmenü-Löschen, Duplizieren und das Ziehen mit der Maus wirken auf die ganze Menge — der Zieh-Geist zeigt dann „3 Objekte“ statt eines Namens.\n\nZiehen: Greife einen Eintrag und zieh ihn auf einen Sidebar-Ort, ein Album oder in „Programme“ — das Ziel leuchtet auf, kurzes Verharren öffnet es (Spring-Loading). AppImages landen per Zug in Programme und sind damit installiert.\n\nGalerie: Bildordner zeigen Kacheln statt Zeilen, Alben (Unterordner) als Karten darüber. Das Raster passt seine Spaltenzahl der Fensterbreite an. Ein Album namens „Hintergrundbilder“ ist besonders: Sein Inhalt erscheint in den Einstellungen unter Hintergrund.\n\nInformationen: Rechtsklick → „Informationen“ zeigt Details (Art, Größe, Geändert, Ort) als schwebende Karte — Klick daneben oder Esc schließt sie.\n\nUmbenennen: Eintrag wählen und Enter drücken. Wer die Dateiendung ändert, wird sicherheitshalber noch einmal gefragt.",
            },
            Article {
                title: "Controller & Bedienung",
                teaser: "Xbox- und PlayStation-Controller steuern das ganze System.",
                content: "Matrix spricht vier Bedienungsarten: Maus, Tastatur, Touch — und seit R92 Controller. Einfach einen Xbox-, PlayStation- oder anderen Controller per USB oder Bluetooth verbinden; Matrix Pad übernimmt sofort, ohne Einrichtung.\n\nDie Belegung folgt der Konsolen-Grammatik: Das Steuerkreuz bewegt den Fokus (die farbige Umrandung wandert durch die Bedienelemente), A/Kreuz wählt aus, B/Kreis geht zurück. Der linke Stick führt den Mauszeiger, der rechte scrollt, L1 und R1 sind Links- und Rechtsklick, Start/Options öffnet die Apps.\n\nWeil die Übersetzung unterhalb des Systems ansetzt, funktioniert sie überall gleich — in jeder Matrix-App und auch in Fremd-Fenstern.\n\nEinstellungen → Bedienung zeigt verbundene Controller, die komplette Belegung und das Zeiger-Tempo; dort wohnen auch Tastatur-, Maus- und Trackpad-Werte.",
            },
            Article {
                title: "Matrix Markdown",
                teaser: "Notizen und Doku: .md lesen und schreiben, mit Live-Vorschau.",
                content: "Matrix Markdown öffnet und bearbeitet Markdown-Dateien (.md) — Doppelklick in Matrix Dateien genügt.\n\nZwei Modi: Lesen zeigt den gerenderten Text (Überschriften, Listen, Tabellen, Bilder, Links, Code-Blöcke mit Sprach-Etikett und Syntaxfarben). Bearbeiten zeigt den Quelltext; beim Zurückwechseln wird automatisch gespeichert.\n\nBilder: Lokale Pfade werden direkt angezeigt; Bilder aus dem Netz bleiben aus Prinzip draußen (ein Chip zeigt die Adresse).\n\nLinks öffnen im Matrix-Browser. Und wie überall gilt: Die Farben kommen aus der lebenden Palette — auch der Code-Block färbt mit dem System um.",
            },
        ],
    },
    Category {
        name: "Der Wächter",
        article: &[
            Article {
                title: "Was ist der Wächter?",
                teaser: "Die Instanz, die über das ganze System wacht — auch vor dem Login.",
                content: "Der Wächter ist die Schutz-Instanz von Matrix. Er gehört zu keiner App und zu keiner Sitzung — er wacht über das GESAMTE System, auch dort, wo du noch gar nicht angemeldet bist: auf dem Login-Bildschirm.\n\nDu erkennst ihn an seiner Stimme: tiefe, gedämpfte Klänge, die bewusst auffallen. Wird dein USB-Login-Schlüssel erkannt, spielt der Wächter seine Melodie — hörbar für den ganzen Raum. Das ist Absicht: Sollte jemand deinen Schlüssel unbefugt benutzen, bekommt es die Umgebung wenigstens mit. Sicherheit, die man hören kann.\n\nDer Wächter hat zwei Aufgaben: Er macht Sicherheits-Ereignisse hörbar (der Schlüssel-Klang am Login-Screen lässt sich deshalb nicht abschalten) — und er hilft dir, wenn du ausgesperrt bist (siehe „Passwort vergessen“).",
            },
            Article {
                title: "Passwort vergessen",
                teaser: "Kein Gerät wird unbrauchbar — der Wächter führt in die Wiederherstellung.",
                content: "Hast du dein Passwort vergessen (und keinen USB-Schlüssel), erscheint der Wächter auf dem Login-Bildschirm — mit einem eigenen, unüberhörbaren Ruf.\n\nEr bietet dir die Wiederherstellung an: eine geführte Umgebung wie die Wiederherstellungs-Umgebung des Leitbilds. Dort wird Schritt für Schritt erklärt, was passiert:\n\n1. Du wählst, was zurückgesetzt wird — nur dein Konto (wenn mehrere Personen den PC nutzen, bleiben die anderen unberührt) oder das ganze System (Werkseinstellung).\n\n2. VOR der Löschung zeigt dir der Wächter ein Speicher-Diagramm: wie viele GB Bilder, Musik, Dokumente, Videos und Sonstiges zu diesem Konto gehören und gelöscht werden. Du weißt genau, was du aufgibst.\n\n3. Erst nach deiner ausdrücklichen Bestätigung wird gelöscht — und der Wächter legt eine Sicherheits-Wartezeit von 30 Minuten ein (dazu gleich mehr). Danach wird vollständig und sauber gelöscht.\n\n4. Danach geht das Setup direkt weiter: Du (oder die Person, die den PC übernimmt) legst ein frisches Konto an und meldest dich neu an.\n\nWichtig zu wissen: Die Wiederherstellung kann Daten LÖSCHEN, aber niemals lesen oder retten. Ein vergessenes Passwort öffnet keine Hintertür zu deinen Dateien.",
            },
            Article {
                title: "Schutz vor fremdem Zugriff",
                teaser: "Eine Wartezeit hält Spaßvögel ab — dein Schlüssel lässt dich sofort durch.",
                content: "Damit nicht einfach ein Freund (oder wer auch immer kurz allein am PC ist) über \u{201e}Passwort vergessen\u{201c} dein Konto löscht, legt der Wächter vor jeder Löschung eine Sicherheits-Wartezeit von 30 Minuten ein.\n\nWährend dieser Zeit läuft ein sichtbarer Countdown — und der Wächter ruft regelmäßig laut. Niemand harrt eine halbe Stunde neben einem rufenden Rechner aus, nur um Unfug zu treiben. Die Löschung wird also nicht verhindert (kein Gerät wird unbrauchbar), sondern nur verzögert und laut angekündigt.\n\nUnd du als Besitzer? Steck deinen USB-Login-Schlüssel ein. Der Wächter prüft sein Geheimnis, erkennt dich als rechtmäßigen Besitzer und löscht sofort — ganz ohne Wartezeit. So bist du in Sekunden durch, während ein Fremder die volle halbe Stunde mit lautem Ruf überstehen müsste.\n\nKurz: Die Wartezeit kostet dich nichts (du hast ja den Schlüssel), hält aber Gelegenheits-Unfug wirksam ab. Und weil die Frist im geschützten System-Werkzeug sitzt, lässt sie sich nicht mit einem Neustart oder Uhr-Umstellen austricksen.",
            },
            Article {
                title: "Die Philosophie dahinter",
                teaser: "Geräte werden nicht unbrauchbar — Daten werden korrekt gelöscht.",
                content: "Manche Systeme machen ein Gerät praktisch wertlos, sobald ein Passwort verloren geht. Das ist nicht unsere Vorstellung von Sicherheit.\n\nDie Matrix-Philosophie: Ein vergessenes Passwort darf niemals ein Gerät unbrauchbar machen. Der PC gehört dem Menschen davor — nicht dem Passwort.\n\nGleichzeitig gilt: Deine persönlichen Daten gehören DIR. Deshalb ist der einzige Weg zurück ins System die KORREKTE, vollständige Löschung der personenbezogenen Daten, die dieses Konto erstellt hat — Bilder, Musik, Dokumente, Einstellungen, Schlüssel. Wer den PC ohne Passwort übernimmt, bekommt ein frisches System, aber niemals deine Dateien.\n\nDass Matrix das kann, liegt an seiner Bauart: Das Betriebssystem selbst ist ein unveränderliches, versioniertes Abbild — es muss nicht neu installiert werden, es IST nach der Löschung der Nutzerdaten wieder wie am ersten Tag. Werkseinstellung heißt bei Matrix: dieselbe frische Installation, drei Minuten statt drei Stunden.\n\nUnd damit Übernahme nie heimlich passiert, ist der Wächter hörbar: Wer die Wiederherstellung startet oder einen Login-Schlüssel benutzt, tut das mit Ansage — für alle im Raum.",
            },
        ],
    },
    Category {
        name: "Design",
        article: &[
            Article {
                title: "Das Designsystem",
                teaser: "Tokens statt Werte: Abstände, Radius, Schrift — alles aus einer Quelle.",
                content: "Kein Wert in einer MatrixKit-App ist erfunden — alles verweist auf Tokens, die aus der System-Shell extrahiert wurden:\n\nAbstände: 2 / 4 / 8 / 12 / 16 / 24 — alles im 4er-Raster. Die Fensterlücken der Leinwand (16) liegen auf demselben Raster.\n\nRundung: EIN Eckenradius (12) für Karten und Knöpfe; Kreise sind halbe Höhe.\n\nSchrift: Inter Variable in vier Größen (12/14/16/20). Hierarchie entsteht durch Größe und Farbrolle, nicht durch Fettdruck.\n\nFarben: nur Material-Rollen (primary, surface, outline …), niemals Hex-Werte im Code — sonst bräche der Wallpaper-Wechsel.\n\nZustände: Interaktive Flächen legen beim Hover 12 % und beim Drücken 20 % der Textfarbe über ihren Grund — dadurch funktioniert Feedback auf jeder Fläche, hell wie dunkel.",
            },
            Article {
                title: "Bewegung",
                teaser: "Federn statt Timer: Warum sich Matrix-Apps lebendig anfühlen.",
                content: "Das Leitbild-Gefühl steckt in der Bewegung, nicht in der Optik — deshalb war die Animations-Engine das erste Fundament von MatrixKit.\n\nWerte springen nie: Die Balken im Systemmonitor FEDERN zu ihrem neuen Ziel und dürfen dabei leicht überschwingen. Die Federn sprechen Leitbild- Sprache — drei Charaktere aus dem echten Leitbild-SDK: glatt (kein Nachschwung), zackig (ein Hauch — der Standard für Panels und Dialoge) und federnd (deutlich, für Akzente). Kommt ein neuer Messwert, während die Feder noch schwingt, nimmt sie einfach das neue Ziel — unterbrechbar, wie im Touch-Leitbild.\n\nDie Kurven und Dauern (200/450/600 ms, sanftes Aus- und Einklingen, Überschwinger für räumliche Bewegung) stammen aus dem Material-3-Bewegungssystem der Shell — Apps und System bewegen sich dadurch im selben Takt.\n\nUnd im Ruhezustand? Kostet eine MatrixKit-App null Animations-Zyklen — getickt wird nur, solange sich etwas bewegt.",
            },
            Article {
                title: "Lebende Icons",
                teaser: "App-Icons, die aus deiner Palette gerendert werden — bei jedem Wechsel neu.",
                content: "MatrixKit-Apps liefern keine statischen Icon-Bilder aus. Ihre Icons werden vom Werkzeug matrixkit-icons aus der aktuellen Palette GEZEICHNET — nach dem Leitbild-Prinzip als getrennte Ebenen: eine gemeinsame Kachel (echte Superellipse mit Lichtkante und Tiefenverlauf) und eine Glyphe pro App mit weichem Schatten.\n\nBei jedem Wallpaper- oder Hell/Dunkel-Wechsel entstehen alle Icons neu — in wenigen Sekunden tragen Launcher und Dock die neuen Farben, auch bei laufenden Apps.\n\nAus der Ebenen-Trennung entstehen Stile: Standard (farbig) und Getönt (monochrom, wie im Leitbild). Umschalten: Datei ~/.config/matrix/icon-style mit Inhalt „tinted“ anlegen; löschen bringt die Farben zurück.\n\nFür Entwickler: Ein neues App-Icon ist eine Zeile in der ICONS-Liste plus eine kleine Glyphen-Funktion — Kachel, Schatten, Größen, Stile und das Umfärben zur Laufzeit übernimmt der Rahmen. Eine App darf stattdessen ein gemaltes Bild mitbringen (eingebettetes PNG in der ICONS-Zeile, wie Rabbit): dann bleibt es in jeder Palette so, wie es gezeichnet wurde.",
            },
        ],
    },
    Category {
        name: "Entwickler",
        article: &[
            Article {
                title: "Architektur",
                teaser: "Vier Crates, klare Rollen: theme, widgets, icons, apps.",
                content: "Der MatrixKit-Workspace besteht aus vier Bausteinen:\n\nmatrixkit-theme — die Wahrheit: Design-Tokens, die Live-Palette (mit Watcher für Wallpaper-Wechsel), die Feder/Bezier-Bewegungs-Engine und die bindende Rechteverwaltung.\n\nmatrixkit-widgets — das Vokabular: Kopfzeile, Fensteraufbau mit nativen Resize-Griffen, Schalter, Fußzeile, Root-Ebene, Zeitgeber. Ein Widget trifft keine eigenen Designentscheidungen — es referenziert Tokens.\n\nmatrixkit-icons — die lebenden Icons: rendert alle App-Icons aus der Palette (tiny-skia, pures Rust).\n\napps/ — die Apps selbst: Monitor, Farben, Klänge, Codes, Schlüssel, Hilfe. Eine App ist im Idealfall nur noch Zustand, Update-Logik und Inhalt.\n\nAlles ist Rust (iced 0.14) — dieselbe Sprache wie der Compositor. Gebaut wird mit cargo build --release im Workspace; die CI testet und baut bei jedem Push.",
            },
            Article {
                title: "Eine neue App bauen",
                teaser: "Vom leeren Ordner zur App in Launcher und Dock — die Schritte.",
                content: "1. Crate anlegen unter apps/<name> und im Workspace-Cargo.toml als Mitglied eintragen. Abhängigkeiten: iced, matrixkit-theme, matrixkit-widgets.\n\n2. Fenster bauen: iced::application + mkw::window_settings(app_id, breite, hoehe) — die App-ID MUSS dem Namen der .desktop-Datei entsprechen, sonst zeigt das Dock ein Fremd-Icon. Die Ansicht liefert mkw::app_window(title, palette, inhalt, …) — Kopfzeile, Griffe und Root-Ebene kommen mit.\n\n3. Live-Farben: Palette::load() beim Start, PaletteWatcher + mkw::tick (2 s) im Update — niemals iced::time::every (braucht Tokio, das wir nicht fahren).\n\n4. Rechte deklarieren: nur die, die die App wirklich nutzt — und JEDEN Zugriff vorher mit rechte.erlaubt(...) prüfen. Bindend heißt bindend.\n\n5. Icon-Rezept in matrixkit-icons ergänzen (eine Zeile + Zeichen-Funktion), Desktop-Eintrag schreiben, App ins Containerfile aufnehmen.\n\n6. Nach dem Anlegen des Desktop-Eintrags liest Matrix Start den neuen Eintrag beim nächsten Öffnen automatisch ein.",
            },
            Article {
                title: "Regeln & Lektionen",
                teaser: "Die hart verdienten Erkenntnisse — damit sie niemand neu bezahlen muss.",
                content: "App-ID = Name der .desktop-Datei. Ohne sie kann das Dock das Fenster keinem Eintrag zuordnen.\n\nNiemals Hex-Farben im App-Code — nur Palette-Rollen. Sonst bricht der Wallpaper-Wechsel.\n\nZeit über mkw::tick (Thread + Kanal), nicht über iced::time::every — funktioniert in Fenster- UND Layershell-Laufzeit.\n\nIcons brauchen versionierte Dateinamen: Die Shell cached Bilder pro Pfad; gleicher Name = altes Bild bis zum Neustart. matrixkit-icons erledigt das automatisch.\n\nWer dms-colors.json liest, muss Schreib-Wettläufe abfangen — der Farb-Sync feuert, während matugen noch schreibt (Palette::load_settled nutzen).\n\nJedes Diagramm trägt eine Beschriftung — räumliche Nähe erzeugt sonst falsche Zuordnung.\n\nAlles Interaktive in Titel/Navigation zeigt Hover-Feedback; Statusmeldungen gehören in die Rahmen-Fußzeile.\n\nUnd die wichtigste: Neue Erkenntnisse gehören in Doku und Rahmen — nicht nur in den Kopf.",
            },
        ],
    },
];
