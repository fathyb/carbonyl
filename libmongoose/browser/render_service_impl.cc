#include "carbonyl/src/browser/render_service_impl.h"

#include <iostream>

#include "base/files/file_path.h"
#include "base/logging.h"
#include "base/memory/read_only_shared_memory_region.h"
#include "base/path_service.h"
#include "carbonyl/src/browser/renderer.h"

namespace mongoose {

namespace {
  // Global Mongoose handle, initialized once
  MongooseHandle g_mongoose_handle = nullptr;

  void EnsureMongooseInitialized() {
    if (!g_mongoose_handle) {
      // Get user data directory for Mongoose storage
      base::FilePath data_dir;
      base::PathService::Get(base::DIR_HOME, &data_dir);
      data_dir = data_dir.AppendASCII(".config").AppendASCII("mongoose");

      std::string data_dir_str = data_dir.AsUTF8Unsafe();
      g_mongoose_handle = mongoose_initialize(data_dir_str.c_str());

      if (!g_mongoose_handle) {
        LOG(ERROR) << "Failed to initialize Mongoose";
      } else {
        LOG(INFO) << "Mongoose initialized with data_dir: " << data_dir_str;
      }
    }
  }
}

MongooseRenderServiceImpl::MongooseRenderServiceImpl(
    mojo::PendingReceiver<mojom::MongooseRenderService> receiver):
    receiver_(this, std::move(receiver)),
    mongoose_handle_(nullptr)
{
  EnsureMongooseInitialized();
  mongoose_handle_ = g_mongoose_handle;
}

MongooseRenderServiceImpl::~MongooseRenderServiceImpl() = default;

void MongooseRenderServiceImpl::DrawText(std::vector<mojom::TextDataPtr> data) {
    std::vector<carbonyl::Text> mapped;

    for (auto& text: data) {
        mapped.emplace_back(text->contents, text->bounds, text->color);
    }

    carbonyl::Renderer::GetCurrent()->DrawText(mapped);
}

void MongooseRenderServiceImpl::CheckNetworkRequest(
    mojom::NetworkRequestDataPtr request,
    CheckNetworkRequestCallback callback) {

  auto result = mojom::BlockResult::New();
  result->should_block = false;
  result->matched_filter = std::nullopt;

  if (!mongoose_handle_) {
    LOG(ERROR) << "CheckNetworkRequest: Mongoose not initialized";
    std::move(callback).Run(std::move(result));
    return;
  }

  // Call Rust FFI to check if request should be blocked
  bool should_block = false;
  char* filter = nullptr;

  std::string url_str = request->url.spec();
  std::string initiator_str = request->initiator.Serialize();

  bool success = mongoose_check_network_request(
      mongoose_handle_,
      url_str.c_str(),
      initiator_str.c_str(),
      request->resource_type,
      &should_block,
      &filter
  );

  if (success) {
    result->should_block = should_block;
    if (filter != nullptr) {
      result->matched_filter = std::string(filter);
      mongoose_free_string(filter);
    }

    if (should_block) {
      LOG(INFO) << "Mongoose: Blocked " << url_str
                << " (filter: " << result->matched_filter.value_or("unknown") << ")";
    }
  } else {
    LOG(ERROR) << "Failed to check network request with Mongoose";
  }

  std::move(callback).Run(std::move(result));
}

void MongooseRenderServiceImpl::DrawImageBitmap(mojom::BitmapDataPtr bitmap) {
  if (!mongoose_handle_) {
    LOG(ERROR) << "DrawImageBitmap: Mongoose not initialized";
    return;
  }

  // Map the shared memory region to access pixel data
  base::ReadOnlySharedMemoryMapping mapping = bitmap->pixels.Map();
  if (!mapping.IsValid()) {
    LOG(ERROR) << "DrawImageBitmap: Failed to map shared memory";
    return;
  }

  // Get pointer to RGBA pixel data
  const uint8_t* pixel_data = static_cast<const uint8_t*>(mapping.memory());
  size_t expected_size = bitmap->width * bitmap->height * 4;  // RGBA = 4 bytes per pixel

  if (mapping.size() < expected_size) {
    LOG(ERROR) << "DrawImageBitmap: Shared memory too small ("
               << mapping.size() << " < " << expected_size << ")";
    return;
  }

  // Call Rust FFI to render the bitmap as ASCII art
  bool success = mongoose_render_bitmap(
      mongoose_handle_,
      pixel_data,
      bitmap->width,
      bitmap->height,
      bitmap->bounds.x(),
      bitmap->bounds.y()
  );

  if (!success) {
    LOG(ERROR) << "Failed to render bitmap with Mongoose";
  } else {
    LOG(INFO) << "Mongoose: Rendered " << bitmap->width << "x" << bitmap->height
              << " bitmap at (" << bitmap->bounds.x() << ", " << bitmap->bounds.y() << ")";
  }
}

void MongooseRenderServiceImpl::SetHybridAsciiMode(bool enabled) {
  if (!mongoose_handle_) {
    LOG(ERROR) << "SetHybridAsciiMode: Mongoose not initialized";
    return;
  }

  mongoose_set_ascii_mode(mongoose_handle_, enabled);
  LOG(INFO) << "Hybrid ASCII mode " << (enabled ? "enabled" : "disabled");
}

void MongooseRenderServiceImpl::SetTitle(const std::string& title) {
  // Forward to existing renderer
  carbonyl::Renderer::GetCurrent()->SetTitle(title);
}

}  // namespace mongoose
