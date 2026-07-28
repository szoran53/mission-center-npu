# Session Handoff - AMD NPU Integration and Layout Consistency

## Accomplished Tasks
1. **AMD NPU Details Screen Crash Fix**:
   - Added the GObject property `infobar_content` to `PerformancePageNpu` returning `None` initially to prevent lookups from panicking when clicking the sidebar row.
2. **AMD NPU Detection & Permission Fallbacks**:
   - Added checking for both host path `/dev/accel/accel0` and Flatpak sandbox path `/dev/accel0`.
   - Added `npu_present` tracker to cache and return NPU telemetry (such as IRQ rate) even if debugfs permissions are restricted for non-root users.
3. **NpuDetails Sidebar Panel Integration**:
   - Built a dedicated `$NpuDetails` blueprint sidebar layout (`npu_details.blp`/`npu_details.ui`).
   - Built `NpuDetails` Rust GObject backing class (`npu_details.rs`).
   - Cleaned up inline bottom details from the main page (`npu.blp`) and bound static/dynamic telemetry updates to the new sidebar details widget.
4. **Fallback Telemetry Retrieval**:
   - Implemented direct world-readable sysfs parsing fallback in `query_static_info` to retrieve `vbnv` (device name) and `fw_version` (firmware version).
   - Parsed stdout/stderr of `xrt-smi --version` directly to extract XRT build version safely without requiring direct device access or lock acquisitions.
5. **Committed & Pushed Changes**:
   - Changes committed and pushed to `main` branch of submodule `subprojects/magpie` (linked to `szoran53/gng-npu.git`).
   - Changes committed and pushed to `main` branch of `szoran53/mission-center-npu.git`.

## Status
- Compilation builds cleanly and successfully.
- Telemetry shows up completely without any crashes.
- Repository is clean.
