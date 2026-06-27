# NovasphereX Vision

## Core Identity

A NovasphereX egy saját fejlesztésű, Rustban írt, natívan bootolható operációs rendszer.

A projekt célja nem egy meglévő rendszer újracsomagolása, hanem egy saját kernelre, saját rendszerarchitektúrára és saját vizuális identitásra épülő operációs rendszer létrehozása.

A NovasphereX hosszú távú célja egy futurisztikus, mégis retro hangulatú rendszer: olyan, mintha a 16-bites korszak kapott volna egy alternatív, fejlettebb operációs rendszert.

## Visual Direction

A NovasphereX későbbi grafikus felülete pixel art alapú lesz.

A vizuális világ a 16-bites konzolkorszakot idézi, különösen a Sega Genesis / Mega Drive hangulatát:

* éles pixeles formák,
* limitált, tudatosan választott színpaletta,
* sprite-szerű ikonok,
* tile-alapú grafikai elemek,
* erős kontraszt,
* kevés anti-aliasing vagy teljes anti-aliasing mentesség,
* karakteres, bitmap alapú tipográfia.

A cél nem egy modern desktop rendszer retro skinnel, hanem egy olyan GUI, amely alapjaiban pixel art logikával épül fel.

## Rendering Principles

A grafikus rendszer tervezésénél az alábbi elvek elsődlegesek:

* pixel-perfect rendering,
* nearest-neighbor scaling,
* belső logikai felbontás használata,
* egész számú nagyítás,
* bitmap fontok,
* sprite és tile blittelés,
* egyszerű, determinisztikus 2D rajzolási modell,
* alacsony komplexitású compositor,
* palette-conscious grafikai pipeline.

A rendszer grafikus felülete nem vector-first és nem HTML/CSS-szerű modellként indul. Első célként egy saját 2D framebuffer renderer készül, amely képes primitívek, bitmap fontok, ikonok, panelek és később ablakok kirajzolására.

## Resolution Strategy

A NovasphereX grafikus felülete alacsony logikai felbontásra lesz tervezve.

Lehetséges belső célfelbontások:

* 320×180,
* 400×240,
* 640×360.

A tényleges kijelzőfelbontásra a rendszer egész számú skálázással nagyítja majd a képet.

Példák:

* 320×180 → 1280×720 esetén 4× skálázás,
* 400×240 → 1200×720 esetén 3× skálázás,
* 640×360 → 1280×720 esetén 2× skálázás.

Ha a kijelző aránya vagy mérete nem illeszkedik pontosan, a rendszer letterboxing vagy pillarboxing használatával tartja meg a pixel-perfect megjelenést.

## UI Style

A későbbi felhasználói felület főbb stílusjegyei:

* vastag, pixeles ablakkeretek,
* szögletes panelek,
* 8×8, 8×16, 16×16 és 32×32 alapú grafikai rács,
* sprite-szerű gombok,
* ikonikus státuszelemek,
* egyszerű animációk,
* kevés, de karakteres vizuális effekt,
* retro-futurisztikus rendszerhangulat.

A rendszer UI-ja játékos és karakteres lehet, de nem lehet kaotikus. A pixel art stílus mögött mindig tiszta rendszerlogikának és olvashatóságnak kell állnia.

## Typography

A NovasphereX elsődleges tipográfiája bitmap alapú lesz.

Első célok:

* monospaced rendszerfont,
* egyszerű 8×16 vagy 8×8 debug font,
* később saját dekoratív pixel font,
* PSF/BDF jellegű fontformátum támogatása vagy saját egyszerű bitmap fontformátum.

A korai kernel/debug szakaszban a font elsődleges célja az olvashatóság. A későbbi GUI-ban a font már a rendszer vizuális identitásának része lesz.

## Color Philosophy

A NovasphereX nem használ korlátlan, modern UI-színezést alapértelmezésként.

A rendszer palettákban gondolkodik:

* alap rendszerpaletta,
* sötét retro-futurisztikus téma,
* kontrasztos státuszszínek,
* opcionális alternatív témák,
* később dithering támogatás.

A cél a tudatos színhasználat, nem a fotórealisztikus vagy túlzottan finom gradiensalapú megjelenés.

## Graphics Stack Direction

A későbbi grafikus alrendszer tervezett moduljai:

```text
gfx-core
  framebuffer kezelés
  pixel írás
  primitívek
  téglalapok
  vonalak

gfx-blit
  bitmap másolás
  sprite kirajzolás
  tile kirajzolás
  átlátszósági kulcs támogatás

gfx-font
  bitmap font kezelés
  karakter renderelés
  monospace layout

gfx-ui
  panelek
  gombok
  listák
  státuszsávok
  alap widgetek

gfx-wm
  ablakkezelés
  egyszerű compositor
  fókuszkezelés
  kurzor

theme-retro16
  rendszerpaletta
  ikonok
  keretek
  UI sprite-ok

asset-pipeline
  sprite import
  font import
  paletta konverzió
  build-time asset csomagolás
```

## System Design Impact

A pixel art vizuális irány hatással van az operációs rendszer felépítésére is.

A NovasphereX grafikus alrendszere ne abból induljon ki, hogy minden elem tetszőlegesen skálázható és lebegőpontos koordinátákkal működik.

Ehelyett:

* egész koordináták,
* fix rácsok,
* kiszámítható memóriahasználat,
* egyszerű blittelés,
* minimális grafikus absztrakció,
* determinisztikus rajzolási sorrend,
* alacsony overhead.

Ez a megközelítés illeszkedik egy saját kernelhez, ahol az egyszerűség, átláthatóság és kontroll fontosabb, mint a túl korai általánosítás.

## Early Boot Graphics

A korai boot szakaszban a grafika még nagyon egyszerű marad.

Első grafikus célok:

* framebuffer detektálása Limine-on keresztül,
* háttérszín kitöltése,
* boot banner,
* egyszerű pixeles logó,
* debug szöveg bitmap fonttal,
* boot státuszsor.

A korai boot UI már tükrözheti a végleges pixel art irányt, de nem kell teljes GUI-nak lennie.

## Long-Term Experience

A NovasphereX végső felhasználói élménye egy retro-futurisztikus, 16-bites hangulatú operációs rendszer.

A rendszernek olyan érzést kell keltenie, mintha egy alternatív számítástechnikai világban a konzolos pixel art esztétika és a személyi számítógépes operációs rendszerek fejlődése találkozott volna.

A NovasphereX legyen:

* technikailag saját,
* vizuálisan felismerhető,
* egyszerűen érthető,
* alacsony szinten kontrollált,
* játékosan futurisztikus,
* de mérnökileg komoly.

## Guiding Sentence

A NovasphereX nem egy modern OS retro témával.

A NovasphereX egy olyan operációs rendszer, amelyet eleve úgy tervezünk, mintha a pixel art korszak sosem ért volna véget.