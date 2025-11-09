// Copyright 2025 Mongoose Browser Project
// Patch for: mongoose-core/ui/gfx/mongoose_text_capture_device.cc
//
// This file shows the implementation additions needed for hybrid ASCII art rendering.

#include "ui/gfx/mongoose_text_capture_device.h"

#include "base/logging.h"
#include "base/memory/shared_memory_mapping.h"
#include "mojo/public/cpp/system/buffer.h"
#include "third_party/skia/include/core/SkImageInfo.h"
#include "third_party/skia/include/core/SkBitmap.h"

namespace gfx {

// ========================================
// STATIC MEMBER INITIALIZATION
// ========================================

bool MongooseTextCaptureDevice::g_hybrid_ascii_mode_ = false;

// ========================================
// NEW METHOD IMPLEMENTATIONS
// ========================================

// static
void MongooseTextCaptureDevice::SetHybridAsciiMode(bool enabled) {
  g_hybrid_ascii_mode_ = enabled;
  LOG(INFO) << "Mongoose: Hybrid ASCII art mode "
            << (enabled ? "ENABLED" : "DISABLED");
  LOG(INFO) << "  - Images will be rendered as "
            << (enabled ? "high-fidelity chafa art" : "fast Unicode blocks");
}

// static
bool MongooseTextCaptureDevice::IsHybridAsciiModeEnabled() {
  return g_hybrid_ascii_mode_;
}

void MongooseTextCaptureDevice::SetMongooseService(
    mojo::Remote<mongoose::mojom::MongooseRenderService> service) {
  mongoose_service_ = std::move(service);
  LOG(INFO) << "Mongoose: Rendering service connected";
}

void MongooseTextCaptureDevice::onDrawBitmap(
    const SkBitmap& bitmap,
    SkScalar left,
    SkScalar top,
    const SkSamplingOptions& sampling,
    const SkPaint* paint) {

  // Check if we should use hybrid ASCII rendering
  if (g_hybrid_ascii_mode_ && mongoose_service_.is_bound()) {
    // Send bitmap to Rust for high-fidelity ASCII art conversion
    gfx::Rect bounds(static_cast<int>(left),
                    static_cast<int>(top),
                    bitmap.width(),
                    bitmap.height());

    SendBitmapToMongoose(bitmap, bounds);
    bitmaps_sent_++;

    VLOG(2) << "Mongoose: Sent " << bitmap.width() << "x" << bitmap.height()
            << " bitmap for ASCII conversion";
  } else {
    // Use fast block rendering (existing Carbonyl logic)
    bitmaps_skipped_++;

    // TODO: Call existing Carbonyl block rendering code here
    // Example:
    // RenderBitmapAsBlocks(bitmap, left, top);
  }

  // Still call parent implementation for other processing
  SkCanvas::onDrawBitmap(bitmap, left, top, sampling, paint);
}

void MongooseTextCaptureDevice::onDrawImage2(
    const SkImage* image,
    SkScalar left,
    SkScalar top,
    const SkSamplingOptions& sampling,
    const SkPaint* paint) {

  if (g_hybrid_ascii_mode_ && mongoose_service_.is_bound()) {
    // Convert SkImage to SkBitmap
    SkBitmap bitmap = ImageToBitmap(image);

    // Send to Rust
    gfx::Rect bounds(static_cast<int>(left),
                    static_cast<int>(top),
                    bitmap.width(),
                    bitmap.height());

    SendBitmapToMongoose(bitmap, bounds);
    bitmaps_sent_++;
  }

  // Call parent implementation
  SkCanvas::onDrawImage2(image, left, top, sampling, paint);
}

void MongooseTextCaptureDevice::SendBitmapToMongoose(
    const SkBitmap& bitmap,
    const gfx::Rect& bounds) {

  if (!mongoose_service_.is_bound()) {
    LOG(WARNING) << "Mongoose service not bound, skipping bitmap send";
    return;
  }

  // Ensure bitmap is in the right format
  SkImageInfo info = SkImageInfo::MakeN32Premul(bitmap.width(), bitmap.height());

  // Calculate pixel data size
  size_t row_bytes = info.minRowBytes();
  size_t pixel_data_size = info.computeByteSize(row_bytes);

  if (pixel_data_size == 0) {
    LOG(WARNING) << "Invalid bitmap size, skipping";
    return;
  }

  // Read pixels into a buffer
  std::vector<uint8_t> pixel_data(pixel_data_size);
  if (!bitmap.readPixels(info, pixel_data.data(), row_bytes, 0, 0)) {
    LOG(ERROR) << "Failed to read bitmap pixels";
    return;
  }

  // Create shared memory region to send pixels efficiently
  //
  // IMPORTANT: We use shared memory instead of copying the pixel data
  // in the Mojo message. This is critical for performance when sending
  // large images (e.g., 1920x1080 = 8MB of RGBA data).
  mojo::ScopedSharedBufferHandle shared_buffer =
      mojo::SharedBufferHandle::Create(pixel_data_size);

  if (!shared_buffer.is_valid()) {
    LOG(ERROR) << "Failed to create shared buffer";
    return;
  }

  // Map the shared memory and copy pixel data
  mojo::ScopedSharedBufferMapping mapping =
      shared_buffer->Map(pixel_data_size);

  if (!mapping) {
    LOG(ERROR) << "Failed to map shared buffer";
    return;
  }

  memcpy(mapping.get(), pixel_data.data(), pixel_data_size);

  // Create BitmapData struct for Mojo call
  auto bitmap_data = mongoose::mojom::BitmapData::New();
  bitmap_data->pixels = std::move(shared_buffer);
  bitmap_data->width = static_cast<uint32_t>(bitmap.width());
  bitmap_data->height = static_cast<uint32_t>(bitmap.height());
  bitmap_data->bounds = bounds;

  // Send to Rust rendering engine (async call)
  //
  // NOTE: This is an async "fire-and-forget" call. The Rust side will
  // process the bitmap and composite the ASCII art into the frame buffer
  // asynchronously. We don't wait for a response.
  mongoose_service_->DrawImageBitmap(std::move(bitmap_data));

  VLOG(3) << "Mongoose: Sent " << pixel_data_size << " bytes to Rust";
}

SkBitmap MongooseTextCaptureDevice::ImageToBitmap(const SkImage* image) {
  SkBitmap bitmap;

  if (!image) {
    return bitmap;
  }

  SkImageInfo info = SkImageInfo::MakeN32Premul(image->width(), image->height());
  if (!bitmap.tryAllocPixels(info)) {
    LOG(ERROR) << "Failed to allocate bitmap pixels";
    return bitmap;
  }

  if (!image->readPixels(bitmap.pixmap(), 0, 0)) {
    LOG(ERROR) << "Failed to read image pixels";
    bitmap.reset();
  }

  return bitmap;
}

}  // namespace gfx
