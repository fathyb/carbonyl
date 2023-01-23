#ifndef CARBONYL_SRC_BROWSER_SOFTWARE_OUTPUT_DEVICE_PROXY_H_
#define CARBONYL_SRC_BROWSER_SOFTWARE_OUTPUT_DEVICE_PROXY_H_

#include <memory>

#include "base/memory/shared_memory_mapping.h"
#include "base/threading/thread_checker.h"
#include "build/build_config.h"
#include "components/viz/host/host_display_client.h"
#include "components/viz/service/display/software_output_device.h"
#include "components/viz/service/viz_service_export.h"
#include "mojo/public/cpp/bindings/pending_remote.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "services/viz/privileged/mojom/compositing/display_private.mojom.h"
#include "services/viz/privileged/mojom/compositing/layered_window_updater.mojom.h"

#if BUILDFLAG(IS_WIN)
#include <windows.h>
#endif

namespace viz {

// Shared base class for SoftwareOutputDevice implementations.
class SoftwareOutputDeviceBase : public SoftwareOutputDevice {
 public:
  SoftwareOutputDeviceBase() = default;
  ~SoftwareOutputDeviceBase() override;

  SoftwareOutputDeviceBase(const SoftwareOutputDeviceBase&) = delete;
  SoftwareOutputDeviceBase& operator=(const SoftwareOutputDeviceBase&) = delete;

  // SoftwareOutputDevice implementation.
  void Resize(const gfx::Size& viewport_pixel_size,
              float scale_factor) override;
  SkCanvas* BeginPaint(const gfx::Rect& damage_rect) override;
  void EndPaint() override;

  // Called from Resize() if |viewport_pixel_size_| has changed.
  virtual void ResizeDelegated() = 0;

  // Called from BeginPaint() and should return an SkCanvas.
  virtual SkCanvas* BeginPaintDelegated() = 0;

  // Called from EndPaint() if there is damage.
  virtual void EndPaintDelegated(const gfx::Rect& damage_rect) = 0;

 private:
  bool in_paint_ = false;

  THREAD_CHECKER(thread_checker_);
};

// SoftwareOutputDevice implementation that draws indirectly. An implementation
// of mojom::LayeredWindowUpdater in the browser process handles the actual
// drawing. Pixel backing is in SharedMemory so no copying between processes
// is required.
class SoftwareOutputDeviceProxy : public SoftwareOutputDeviceBase {
 public:
  explicit SoftwareOutputDeviceProxy(
      mojo::PendingRemote<mojom::LayeredWindowUpdater> layered_window_updater);
  ~SoftwareOutputDeviceProxy() override;

  SoftwareOutputDeviceProxy(const SoftwareOutputDeviceProxy&) = delete;
  SoftwareOutputDeviceProxy& operator=(const SoftwareOutputDeviceProxy&) = delete;

  // SoftwareOutputDevice implementation.
  void OnSwapBuffers(SoftwareOutputDevice::SwapBuffersCallback swap_ack_callback, gfx::FrameData data) override;

  // SoftwareOutputDeviceBase implementation.
  void ResizeDelegated() override;
  SkCanvas* BeginPaintDelegated() override;
  void EndPaintDelegated(const gfx::Rect& rect) override;

 private:
  // Runs |swap_ack_callback_| after draw has happened.
  void DrawAck();

  mojo::Remote<mojom::LayeredWindowUpdater> layered_window_updater_;

  std::unique_ptr<SkCanvas> canvas_;
  bool waiting_on_draw_ack_ = false;
  SoftwareOutputDevice::SwapBuffersCallback swap_ack_callback_;

#if !defined(WIN32)
  base::WritableSharedMemoryMapping shm_mapping_;
#endif
};

}  // namespace viz

#endif  // CARBONYL_SRC_BROWSER_SOFTWARE_OUTPUT_DEVICE_PROXY_H_
