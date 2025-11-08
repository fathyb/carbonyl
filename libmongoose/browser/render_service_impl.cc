#include "carbonyl/src/browser/render_service_impl.h"

#include <iostream>

#include "base/files/file_path.h"
#include "base/logging.h"
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

  // TODO: Implement network blocking using Mongoose privacy engine
  // For now, allow all requests
  auto result = mojom::BlockResult::New();
  result->should_block = false;
  result->matched_filter = std::nullopt;

  std::move(callback).Run(std::move(result));
}

void MongooseRenderServiceImpl::DrawImageBitmap(mojom::BitmapDataPtr bitmap) {
  // TODO: Implement ASCII art rendering using Mongoose render engine
  // This will extract pixels from shared memory and send to Rust for chafa conversion
  LOG(INFO) << "DrawImageBitmap called (not yet implemented) - "
            << bitmap->width << "x" << bitmap->height;
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
