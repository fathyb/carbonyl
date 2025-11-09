// Copyright 2025 Mongoose Browser Project
// Patch for: mongoose-core/ui/gfx/mongoose_text_capture_device.h
//
// This file shows the additions needed to the existing MongooseTextCaptureDevice
// class to support hybrid ASCII art rendering.

// ========================================
// ADD to existing MongooseTextCaptureDevice class declaration:
// ========================================

#ifndef MONGOOSE_CORE_UI_GFX_MONGOOSE_TEXT_CAPTURE_DEVICE_ADDITIONS_H_
#define MONGOOSE_CORE_UI_GFX_MONGOOSE_TEXT_CAPTURE_DEVICE_ADDITIONS_H_

#include "mojo/public/cpp/bindings/remote.h"
#include "libmongoose/browser/mongoose.mojom.h"
#include "third_party/skia/include/core/SkBitmap.h"
#include "third_party/skia/include/core/SkCanvas.h"
#include "ui/gfx/geometry/rect.h"

namespace gfx {

/// Additions to MongooseTextCaptureDevice for hybrid rendering
///
/// INTEGRATION: Add these methods and members to the existing
/// MongooseTextCaptureDevice class in mongoose_text_capture_device.h
class MongooseTextCaptureDevice : public SkCanvas {
 public:
  // ... existing methods ...

  // ===================
  // NEW METHODS - ADD THESE:
  // ===================

  /// Enable or disable hybrid ASCII art mode
  ///
  /// When enabled, images are captured as raw bitmaps and sent to the Rust
  /// rendering engine for high-fidelity ASCII art conversion using chafa.
  ///
  /// When disabled (default), images are rendered using fast Unicode blocks
  /// for 60fps performance.
  static void SetHybridAsciiMode(bool enabled);

  /// Check if ASCII art mode is currently enabled
  static bool IsHybridAsciiModeEnabled();

  /// Set the Mojo remote for communicating with Rust rendering engine
  ///
  /// This should be called during initialization to establish the connection.
  void SetMongooseService(
      mojo::Remote<mongoose::mojom::MongooseRenderService> service);

 protected:
  // ===================
  // OVERRIDE THESE METHODS:
  // ===================

  /// Override to intercept bitmap drawing
  ///
  /// This is called by Skia whenever an image is drawn on the canvas.
  /// In hybrid ASCII mode, we capture the bitmap and send it to Rust.
  void onDrawBitmap(const SkBitmap& bitmap,
                   SkScalar left,
                   SkScalar top,
                   const SkSamplingOptions& sampling,
                   const SkPaint* paint) override;

  /// Override to intercept image drawing
  void onDrawImage2(const SkImage* image,
                   SkScalar left,
                   SkScalar top,
                   const SkSamplingOptions& sampling,
                   const SkPaint* paint) override;

 private:
  // ===================
  // NEW PRIVATE METHODS:
  // ===================

  /// Send bitmap data to Rust rendering engine via Mojo
  ///
  /// This converts the SkBitmap to raw RGBA pixels, puts them in shared memory,
  /// and sends a DrawImageBitmap Mojo call to the Rust side.
  void SendBitmapToMongoose(const SkBitmap& bitmap,
                           const gfx::Rect& bounds);

  /// Convert SkImage to SkBitmap for processing
  SkBitmap ImageToBitmap(const SkImage* image);

  // ===================
  // NEW MEMBER VARIABLES:
  // ===================

  /// Global flag for hybrid ASCII mode (shared across all instances)
  static bool g_hybrid_ascii_mode_;

  /// Mojo interface to Rust rendering engine
  mojo::Remote<mongoose::mojom::MongooseRenderService> mongoose_service_;

  /// Statistics for debugging
  int bitmaps_sent_ = 0;
  int bitmaps_skipped_ = 0;
};

}  // namespace gfx

#endif  // MONGOOSE_CORE_UI_GFX_MONGOOSE_TEXT_CAPTURE_DEVICE_ADDITIONS_H_
