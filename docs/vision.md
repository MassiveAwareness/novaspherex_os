# NovasphereX Vision

## Core Identity

NovasphereX is a custom-built, natively bootable operating system written in Rust.

The goal of the project is not to repackage an existing operating system, nor to build a conventional Unix-like environment as a first principle. NovasphereX is intended to become an original operating system built around its own kernel, its own system architecture, and its own visual identity.

The long-term direction of NovasphereX is retro-futuristic: a system that feels as if the 16-bit era had continued into an alternate, more advanced computing timeline.

NovasphereX should be technically serious, visually recognizable, and architecturally transparent. It should be a system where low-level control, deterministic behavior, and stylistic cohesion matter from the earliest boot stages to the eventual graphical user interface.

---

## Visual Direction

The future graphical interface of NovasphereX is pixel-art based.

Its visual language is inspired by the 16-bit console era, especially the Sega Genesis / Mega Drive aesthetic. This influence does not mean copying a specific console interface. Instead, it means adopting the underlying visual discipline of that era:

* sharp pixel-defined shapes,
* limited and deliberate color palettes,
* sprite-like icons,
* tile-based graphic elements,
* strong contrast,
* little or no anti-aliasing,
* bitmap-based typography,
* direct and readable visual hierarchy.

The goal is not to create a modern desktop environment with a retro skin. The goal is to design a graphical operating system interface that is built from the ground up around pixel-art logic.

This distinction is important. A retro skin can be applied late. A pixel-art operating system must be designed early.

---

## Rendering Principles

The graphical system should be guided by the following rendering principles:

* pixel-perfect rendering,
* nearest-neighbor scaling,
* internal logical resolutions,
* integer scaling,
* bitmap fonts,
* sprite and tile blitting,
* simple deterministic 2D drawing,
* low-complexity composition,
* palette-conscious rendering,
* predictable memory usage,
* integer coordinates by default.

The NovasphereX graphics model should not begin as a vector-first or HTML/CSS-like layout system. The first graphical foundation should be a custom 2D framebuffer renderer capable of drawing primitives, bitmap fonts, icons, panels, and eventually windows.

The renderer should favor clarity over abstraction. A small, predictable graphics pipeline is preferred over an overly general system that hides its cost or behavior.

---

## Resolution Strategy

NovasphereX should be designed around low internal logical resolutions.

Possible internal target resolutions include:

* `320×180`,
* `400×240`,
* `640×360`.

The actual display resolution should be treated as an output surface. The logical image should be scaled to the physical framebuffer using integer scaling whenever possible.

Examples:

* `320×180 → 1280×720` with `4×` scaling,
* `400×240 → 1200×720` with `3×` scaling,
* `640×360 → 1280×720` with `2×` scaling,
* `640×360 → 1920×1080` with `3×` scaling.

When the physical display resolution does not match the logical aspect ratio exactly, the system should use letterboxing or pillarboxing to preserve pixel-perfect output.

The interface should not depend on arbitrary fractional scaling. Pixel structure is part of the system identity and should remain visible, intentional, and stable.

---

## UI Style

The eventual user interface should use a strong Retro16 design language.

Expected style elements include:

* thick pixel-defined window borders,
* angular panels,
* grid-based layout,
* `8×8`, `8×16`, `16×16`, and `32×32` visual units,
* sprite-like buttons,
* iconic status indicators,
* simple animations,
* minimal but distinctive effects,
* retro-futuristic system atmosphere.

The UI may be playful and characterful, but it must not become visually chaotic. Behind the pixel-art style there must always be clear system logic, readable hierarchy, and predictable interaction.

NovasphereX should feel expressive without becoming noisy.

---

## Typography

NovasphereX should use bitmap-based typography as a core part of its identity.

Initial typography goals:

* monospaced system font,
* simple `8×16` or `8×8` debug font,
* later custom decorative pixel font,
* support for PSF/BDF-like font formats or a custom simple bitmap font format,
* consistent font rendering through the framebuffer graphics stack.

During early kernel and debug stages, the primary purpose of typography is readability. Later, in the GUI stage, typography should become part of the operating system's visual identity.

The system font should be legible, compact, and visually compatible with the Retro16 design language.

---

## Color Philosophy

NovasphereX should not rely on unlimited modern UI color usage as its default visual model.

The system should think in palettes.

Important color concepts include:

* base system palette,
* dark retro-futuristic theme,
* high-contrast status colors,
* optional alternate palettes,
* palette-indexed assets where appropriate,
* future dithering support,
* deliberate color reuse across UI elements.

The goal is intentional color use, not photorealism or overly subtle gradient-based visual design.

Color should communicate system state. It should also reinforce the identity of the operating system.

---

## Graphics Stack Direction

The planned graphical subsystem may evolve into modules similar to the following:

```text
gfx-core
  framebuffer management
  pixel writing
  primitive drawing
  rectangles
  lines
  clipping

gfx-blit
  bitmap copying
  sprite drawing
  tile drawing
  transparency-key support
  raw image surfaces

gfx-font
  bitmap font loading
  character rendering
  monospace layout
  debug text rendering

gfx-ui
  panels
  buttons
  lists
  status bars
  basic widgets
  layout primitives

gfx-wm
  window management
  simple compositor
  focus handling
  cursor handling
  window movement

theme-retro16
  system palette
  icons
  borders
  UI sprites
  decorative elements

asset-pipeline
  sprite import
  font import
  palette conversion
  build-time asset packaging
  raw framebuffer-ready asset generation
```

The existing boot background pipeline is an early proof of this direction. It already demonstrates build-time asset preparation, raw framebuffer-ready image data, nearest-neighbor scaling, and embedded assets.

---

## System Design Impact

The pixel-art visual direction affects more than appearance. It also influences system architecture.

The NovasphereX graphical subsystem should not assume that every visual element is arbitrarily scalable, floating-point based, or layout-driven by complex dynamic styling rules.

Instead, the system should prefer:

* integer coordinates,
* fixed grids,
* deterministic memory use,
* simple blitting,
* minimal graphics abstraction,
* deterministic draw order,
* low overhead,
* predictable rendering cost.

This approach fits a custom kernel environment where simplicity, transparency, and control are more important than premature generality.

The system should be designed so that the renderer can be understood, debugged, and evolved incrementally.

---

## Early Boot Graphics

The early boot stage is the first place where the visual identity of NovasphereX becomes visible.

Originally, the kernel displayed a simple framebuffer banner. As of the current early Retro16 graphics work, the system can embed and display one of several prepared boot backgrounds.

Current early boot graphics capabilities include:

* framebuffer detection through Limine,
* raw framebuffer drawing,
* build-time PNG-to-RGBX asset conversion,
* generated asset validation,
* embedded boot background assets,
* boot-time selection of one background from five candidates,
* nearest-neighbor scaling to the framebuffer,
* serial logging of the selected boot background.

This is still not a full graphical user interface. However, it is an important transition: the boot process no longer merely proves framebuffer access; it now begins to express the intended visual identity of the operating system.

Future early boot graphics goals include:

* debug text over the boot background,
* bitmap font rendering,
* boot status bar,
* pixel-art logo,
* staged boot progress indicators,
* panic screen styling,
* diagnostic overlay mode.

The early boot UI should remain simple, but it should increasingly resemble the final system language.

---

## Boot Background Pipeline

The current boot background pipeline is intentionally build-time oriented.

Source images are stored under:

```text
assets/images
```

The asset preparation script converts them into raw framebuffer-ready data under:

```text
kernel/src/assets/generated
```

The kernel embeds the generated data with `include_bytes!`.

This avoids requiring a filesystem, PNG decoder, allocator, or userspace loader during early boot. It also makes the first graphics pipeline deterministic and simple.

The current model is:

```text
Retro16 PNG source image
  -> build-time conversion
  -> raw RGBX buffer
  -> embedded kernel asset
  -> boot-time background selection
  -> framebuffer drawing
```

This design is temporary but appropriate for the current kernel stage. Runtime asset loading should come later, after the filesystem, memory allocator, and file loading infrastructure exist.

---

## Randomness and Boot-Time Variation

NovasphereX currently uses a lightweight boot-time entropy source for harmless visual variation.

The boot background selection uses a TSC-based seed to select one of five backgrounds in the inclusive range:

```text
1..=5
```

This is not cryptographic randomness and must not be used for security-sensitive decisions.

For visual boot variation, this approach is acceptable. In the future, a real entropy subsystem should provide stronger randomness for security, scheduling, identifiers, and other kernel-level needs.

---

## Color and Asset Constraints

Retro16 assets should be designed with constraints in mind.

Recommended asset guidelines:

* avoid excessive gradients,
* preserve sharp pixel shapes,
* design around a limited palette,
* prefer clear silhouettes,
* keep contrast high enough for overlays,
* avoid visual noise in areas where boot text may later appear,
* use nearest-neighbor-friendly pixel structure,
* maintain consistency across background images.

The boot background should support the operating system identity, not overwhelm the boot diagnostics.

---

## User Experience Direction

The final user experience of NovasphereX should feel like a retro-futuristic 16-bit operating system from an alternate computing timeline.

The system should feel:

* technically custom,
* visually distinctive,
* easy to understand,
* low-level and controlled,
* playful but not unserious,
* futuristic without abandoning pixel-art discipline,
* expressive without becoming chaotic.

NovasphereX should not imitate modern desktop environments too closely. Its interface should grow out of its own renderer, its own constraints, and its own system architecture.

---

## Engineering Philosophy

NovasphereX is not only a visual experiment. It is also a low-level engineering project.

The kernel should remain:

* observable,
* documented,
* incremental,
* explicit about unsafe code,
* transparent about hardware assumptions,
* careful with abstractions,
* stable after each milestone.

Every major subsystem should be introduced through a small proof, then expanded only after the proof is validated.

This philosophy has already shaped the project:

* boot proof before CPU abstraction,
* CPU diagnostics before interrupt complexity,
* software interrupt tests before hardware timer delivery,
* APIC probing before APIC MMIO,
* page-table mapping before APIC timer delivery,
* embedded raw assets before runtime image loading.

The same approach should continue as the memory subsystem, scheduler, filesystem, graphics stack, and user interface evolve.

---

## Long-Term Experience

NovasphereX should eventually become a native Rust operating system with a coherent retro-futuristic personality.

The ideal experience is an operating system that feels as though console-style pixel art aesthetics and personal-computer operating systems evolved together, rather than separately.

It should be possible to recognize NovasphereX visually from a single screenshot.

It should also be possible to inspect the kernel architecture and understand why the system behaves the way it does.

The long-term goal is not only to build something that boots. The goal is to build something with identity.

---

## Guiding Sentence

NovasphereX is not a modern operating system with a retro theme.

NovasphereX is an operating system designed as if the pixel-art era never ended.
