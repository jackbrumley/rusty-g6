# TODO

## Now

### Device "connected" after reboot but controls are dead until replug

Reported 2026-11-03. Happens often after a reboot; unplugging and replugging
the G6 fixes it.

- Status shows **Connected**, but every toggle is off: the device state was
  never actually loaded into the UI.
- **Read Device State** reports "read successfully", yet the toggles still do
  not turn on.
- Toggling output (Speakers/Headphones): the device relay audibly clicks
  **once**, so the first write reaches the hardware, but the UI does not
  update to Headphones. Further clicks do nothing at all, as if the
  device/session is locked up.

**Findings (session log and kernel log of the 2026-11-03 boot):**
- The app opened interface 4 (`/dev/hidraw3`) about 20 s after boot. The node
  stayed the same until the manual replug at 18:31, so the handle was not
  stale and was the right interface.
- Every state query got **no packet at all** (`Received 0 packet(s)` for all
  14 queries, at 18:10 and again at 18:29), and the listener saw no events
  for 20 minutes. Writes succeeded. Interface 4 has only an interrupt IN
  endpoint (`ep_85`), so writes go over the control pipe (SET_REPORT) and all
  answers come back on interrupt IN: the control path worked, the IN path was
  silent.
- Right after the replug, every query was answered within 1 ms on a freshly
  opened handle.
- Every cold boot in the journal (Oct 16, 18, 28, Nov 2, Nov 3) logs 12
  `__uac_clock_find_source(): selector reported illegal value` errors for the
  G6. No later re-enumeration (replug or resume) does. The G6's firmware is
  not fully up when it is enumerated at power-on. Several of those boots are
  followed by an 8-10 s disconnect/reconnect, which is a manual replug.
- "Further clicks do nothing" was an app bug on top of that: the toggle target
  came from the cached output, which never left its default (Headphones), so
  every click re-sent "switch to Speakers". The first click switched the
  relay. Later clicks asked for the output it already had.
- "Read successfully" was wrong too: the read reported success whatever it
  parsed. Its "Refreshing device connection" step only rebuilt the HID
  enumerator, not the open handle. The UI also gave up on the start-up read
  after 1 s while the backend spent 7 s waiting on 14 unanswered queries.
  Device commands ran on the main thread, which froze the window meanwhile.

**Fixed (dev, after v1.0.8):**
- A state read fails (`DeviceUnresponsive`) unless it parsed at least one
  response. It stops after 2 silent queries in a row instead of waiting out
  all 14. Read Device State now reports the failure.
- Self-healing on an unresponsive read, logged with `SELF-HEAL`: close and
  reopen the HID handle, then read again. If the G6 is still silent (Linux),
  reset it on the USB bus (libusb `USBDEVFS_RESET`, the re-enumeration a
  replug causes), reconnect and read again. The USB reset runs at most once
  until a read succeeds, so a dead device is never reset in a loop. A
  remaining failure says to replug.
- Writes are checked for an acknowledgement. An unacknowledged output switch
  re-reads the device (healing it if needed) to learn whether it took effect
  (`UNCONFIRMED WRITE` in the log).
- Toggling output reads the device first unless its output was confirmed by
  a read or a change event since the handle was opened.
- The event listener starts once per app run, so reconnects no longer stack
  up extra reader threads on the same handle.
- Device commands run off the main thread, and the UI waits up to 15 s for a
  read. The backend emits `device-update` when a read finishes.

**Still to confirm on real hardware after a cold boot** (with autostart):
- Does the first read log `SELF-HEAL`, and which step brings the G6 back:
  reopening the handle, or only the USB reset? Search the session log for
  `SELF-HEAL`.
- Does the USB reset interrupt audio noticeably, and does the G6 keep its
  output and effect settings across it?
- If even the USB reset does not help, try a power-cycle-like fix next, e.g.
  toggling `/sys/bus/usb/devices/<port>/authorized` (needs root via the udev
  rule), or delay the first open until the audio stack has configured the
  device.
- Windows has no USB reset step (the HID driver owns the device). Check
  whether the bug happens there at all.

**Outside evidence (linuxblaster_control, 2026-11):** the same cold-boot bug
is documented in `packaging/coldboot/README.md` of
https://github.com/RizeCrime/linuxblaster_control. They found that a
`USBDEVFS_RESET` does wake the HID endpoint, but **leaves the audio interface
unusable until the next full re-enumeration**. That contradicts our in-session
self-heal. Their fix runs once at boot, before PipeWire opens the device:
- A udev `add` rule starts a oneshot service.
- The service resets the G6, waits for it to re-enumerate, then loads
  `snd-usb-audio`, which modprobe holds back from the first enumeration.
- Result: no `__uac_clock_find_source` errors and a working HID endpoint.

Test on hardware after a cold boot: does playback still work after our
`SELF-HEAL` USB reset? If it doesn't, drop the in-session reset (keep the
handle reopen) and offer a boot-time helper like theirs instead, as an
explicit install step with its side effects listed.

### Check the protocol refactor on hardware (2026-11-15)

The device layer was restructured (one effect model, one parse path, every
write answered by the device). Behaviour that changed and needs a check on
the G6:
- The controls show what the G6 answers, not what was clicked. A write the
  G6 does not answer now shows an error toast instead of failing silently.
- The Bass level slider now reaches the device (`96:19`); before, only the
  on/off switch was sent.
- Mic boost is read from the device and always shown on the Input page (the
  Experimental switch is gone).
- An EQ preset or SBX switch can take up to about half a second, because
  every answer the G6 sends is read and applied.

### Microphone very quiet on Linux

Reported 2026-11-15. The G6 mic is barely audible on Linux, and a manual
`amixer` fix is lost at every reboot.

**Findings (read-only, 2026-11-15):**
- The G6 records from **Line In**, not the mic: `PCM Capture Source` = Line In.
  The External Mic capture level is fine (0 dB), so raising volumes barely
  helps.
- PipeWire's only G6 input port is Line In. Its ALSA card profile path
  `analog-input-linein.conf` forces `PCM Capture Source:Line In`, and its mic
  path does not know the G6's `External Mic` item, so it never offers a mic
  port.
- `/var/lib/alsa/asound.state` stores Line In and `alsa-restore` puts it back
  at boot. So the kernel default, alsa-restore and PipeWire can each undo a
  manual fix.
- `configure_microphone` (`app/commands/system.rs`) shells out to `amixer`,
  runs only from the experimental "Setup Mic" button, is not stored, and its
  toast names the wrong source ("Digital Input (S/PDIF)"; it is "Analog
  Stereo"). It logs with `eprintln!`.

**Built (dev, 2026-11-15), to test on hardware:** `g6_mixer/` keeps the
stored recording source (`settings.json`, default External Mic) selected. It
applies it when the G6's sound card appears and again whenever another
program changes it. If something switches it back more than 5 times in a
minute, it stops and shows an error. The Input page has a Recording source
picker; the old `amixer` "Setup Mic" button is gone. Check after a reboot:
the mic is loud without touching anything, and the session log shows
`RECORDING_SOURCE` lines. Mic level and mic boost read-back are still to do.

**Hardware findings (2026-11-15):**
- WirePlumber (0.4.17) writes `PCM Capture Source = Line In` in a short
  burst (about 6 writes in 2 s) whenever it starts or re-activates the G6's
  input (its ACP "Line In" path). The first watch re-applied after every
  write, hit its 5-per-minute limit and stopped.
- Fix (in `g6_mixer/watch.rs`): wait for 1.5 s of quiet, then re-apply once.
  Tested: a WirePlumber restart now ends on External Mic with one re-apply.
- A per-user WirePlumber rule with `api.alsa.soft-mixer = true` did **not**
  help: the port activation still writes Line In. Not needed; removed.
- WirePlumber renames the G6's nodes (`.2`, `.3` ...) when it sees a
  duplicate during a replug or profile switch, then cannot find the default
  it remembered by name and falls back to another output. A clean restart
  with the stale names removed from `~/.local/state/wireplumber/default-nodes`
  fixed it on the maintainer's machine. Worth a line in the docs for users.

**Original plan:**
- A Linux-only mixer module on the `alsa` crate (model: linuxblaster_control
  `src/mixer.rs`, MIT). Store the recording source in `settings.json`
  (default External Mic) and apply it on every connect.
- Watch `PCM Capture Source` with ALSA control events and re-apply it if
  PipeWire switches it back; log each re-apply.
- Replace `configure_microphone` and its `amixer` calls.
- Add `libasound2-dev` to the Containerfile and the Linux CI job.
- Input page: recording source picker, mic level, and mic boost read back
  from the device (`5a 3c 02 01 00`).
- Later, optional: ship an ALSA card profile path so PipeWire offers a real
  mic port even without the app (needs root; the installers already add a
  udev rule).

Test first, so we know whether the watch is needed:
`amixer -c G6 cset name='PCM Capture Source' 'External Mic'`, check the mic,
then `systemctl --user restart wireplumber pipewire` and
`amixer -c G6 cget name='PCM Capture Source'`. If it is back on Line In,
PipeWire resets it.

## Next

### More G6 controls (from Creative's app and linuxblaster)

Sources:
- linuxblaster_control's `UsbProtocol.md`: tested on hardware, MIT licence,
  with 42 USB captures in `sniffer/captures/`.
- A decompile of Creative's Sound Blaster Command 3.4.98. Its G6 UI uses
  these features: SBX effects, graphic EQ, voice morph, mic EQ, noise
  reduction, echo cancellation, mic smart volume, acoustic engine, virtual
  speaker config and HRTF, headphone gain, Direct and S/PDIF direct, the
  Dolby decoder, the LED indicator, lighting, and the UAC1/UAC2 mode.

Use their findings, write the code in our own structure, and credit
linuxblaster_control in the README with the first feature that uses them.
Every command below is backed by a linuxblaster capture or a hardware test
they wrote down. Re-check each one on our G6 before it ships.

Order (each item: protocol builder + parser, state read, setter, Tauri
command, UI):
1. ~~**Split first.**~~ Done: `g6_device/` (connection, transport,
   setters) and `g6_protocol/` (parser, reads, writes, equalizer).
2. **Graphic EQ.** Built (toggle, presets, draggable curve, preamp), not yet
   tested on hardware. Check: each write takes effect, the curve follows
   output switches (multi-feature `0x11` reports), presets apply in one go.
   Original plan: Toggle `96:09`, preamp `96:0a` (±6 dB, step 0.5), bands
   `96:0b`–`96:14` (31 Hz–16 kHz, ±12 dB, step 0.5). Read
   `5a 11 03 01 96 [id]`, write `5a 12 07 01 96 [id] [f32 LE]`. Replace the
   unused `equalizer` and `extended_params` fields in `g6_spec.rs`: nothing
   fills them, and `0x0a`–`0x14` are the EQ, not "extended params". The EQ
   also sets bit 0 of the `0x26` status. Captures: `set_eq_on`, `set_eq_off`,
   `set_eq_62hz_0db`.
3. **DAC filter.** Built (Output page dropdown, all five filters, read at
   start-up), not yet tested on hardware. Bytes confirmed against
   linuxblaster's `set_filter_*` captures; NOS (`0x03`) is listed by the
   device but its write is not captured, so check NOS first.
4. **Recording controls.** Built on the Mic tab, not yet tested on hardware:
   noise reduction and level, echo cancellation `95:00`, mic smart volume
   `95:2c` and the mic EQ switch `95:13` (all captured: linuxblaster and
   soundblaster-x-g6-cli `payloads/raw`). The mic level slider is built too. Still to do: mic EQ presets on
   `95:14`-`95:1b` (captured).
5. **Smaller items:**
   - `96:17` (10–300, step 0.5): linuxblaster calls it surround distance,
     but Creative's own parameter table names it the **XBass crossover**.
     Check on hardware which one it is before labelling it.
   - ~~Decoder mode `97:02`~~ built (Sound tab).
   - Smart Volume mode `96:06` (0 = Normal, 1 = Loud, 2 = Night). The
     `SmartVolumePreset` enum exists, but nothing reads or writes it yet.
6. **Lighting.** On/off and static colour built (Device tab). Still to do:
   effects (pulsate, cycle, music reactive), not captured. Original notes: On/off `5a 3a 02 06 [0|1]`; colour `5a 3a 06 04 00 03 01 00 01`
   then `5a 3a 09 0a 00 03 01 01 ff B G R`. Read with `5a 3a 01 07` and
   `5a 3a 06 0b 01 00 01 01 01`.
7. **Mixer and recording source.** These are standard USB Audio Class
   controls (ALSA card `G6`), not HID. Decide whether Rusty G6 should show
   them at all.
8. **Features decoded from Creative's app, not yet captured.** Decoded
   statically on 2026-11-15 from `Platform/CTHIDRpA.dll`, which builds the
   frames; `SndCrUSB.DLL` only passes parameter indexes to it. Raw dumps are
   in `temp/sbc-decode/` (untracked). Each one still needs a capture or a
   reversible hardware test before it ships.
   - ~~**Voice morph**~~: shipped on the Mic tab (switch and 18 presets).
     A read on a G6 (2026-11-16) answered all of `95:0a`–`12` inside the
     table's ranges. Still to confirm by ear: switching it on changes the
     voice. Not built yet: sliders for the eight values (see
     `docs/usb-protocol.md`).
   - **Mic reverb:** `95:2e`–`35` plus family `9f` ids `01`–`1f`. This is the
     only "EAX" on the G6: there is no playback reverb.
   - **Mic EQ band frequency and width:** `95:1c`–`23` and `95:24`–`2b`.
   - **Flag family `0x39`:** set `5a 39 03 00 [SUB] [0|1]`; read
     `5a 39 01 01`, where bit SUB of the reply's LE u32 at offset 4 is the
     state. SUB: `05` stereo direct, `06` headphone high gain, `0a` speaker
     HRTF, `0b` auto sleep, `0d` S/PDIF out direct, `0e` LED indicator, `1a`
     UAC2 only (its read shares bit 15 with `0f`, so don't trust it). Restore
     defaults is `5a 39 03 00 03 01`.
   - **Speaker config:** set `5a 29 05 00 [u32 LE]` (1 = headphones,
     2 = stereo, 4 = 5.1; no 7.1); read `5a 29 01 01`.
   - **Lighting effects:** slot `5a 3a 02 0f S`, pattern `5a 3a 03 08 S P`,
     timing `5a 3a 06 0a S X 03 t t`, restart `5a 3a 01 0c`. Some field
     meanings are still guesses.
   - Several of our "commit" packets (`5a 2c 01 01`, `5a 3c 02 01`,
     `5a 39 01 01`) are really reads. They are harmless, but name them as reads.

Small corrections from the same doc:
- The `0x3c` family is only mic boost, so `AudioConfig` in `g6_spec.rs` is not
  a separate setting.
- Bit 2 of the `0x26` status means voice clarity (noise reduction) is on.

- After the feature sprint: redesign the layout and visual packaging so the
  new controls fit. Use Voquill (`~/Coding/voquill`) as the model.
- App Standard section 6: check the Settings layout (General, Updates,
  app-specific, Debug), modals (`role="dialog"`, Escape, focus) and toast
  timings against the standard.

## Ideas

- None yet.

## Done

### Distribution

- [x] One-line install: `install.sh` (apt/dnf/AppImage; the AppImage path must
      also install the udev rule) and `install.ps1` (per-user setup.exe), served
      from `docs/` at `https://brumley.info/rusty-g6/`.
- [x] One-click in-app update: port Voquill's `install_update` (re-run the
      installer, exit, relaunch; relaunch old version with `--update-failed`).
- [x] Install page at `docs/index.html` and enable GitHub Pages from `main:/docs`.
- [x] Release workflow in GitHub Actions that builds and verifies all five
      assets, so the installer's expected file names can't drift.
