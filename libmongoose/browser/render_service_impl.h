#ifndef MONGOOSE_SRC_BROWSER_RENDER_SERVICE_IMPL_H_
#define MONGOOSE_SRC_BROWSER_RENDER_SERVICE_IMPL_H_

#include "carbonyl/src/browser/export.h"
#include "carbonyl/src/browser/mongoose.mojom.h"
#include "carbonyl/src/browser/ffi_exports.h"
#include "mojo/public/cpp/bindings/pending_receiver.h"
#include "mojo/public/cpp/bindings/receiver.h"

namespace mongoose {

class CARBONYL_RENDERER_EXPORT MongooseRenderServiceImpl: public mojom::MongooseRenderService {
 public:
  explicit MongooseRenderServiceImpl(mojo::PendingReceiver<mojom::MongooseRenderService> receiver);
  MongooseRenderServiceImpl(const MongooseRenderServiceImpl&) = delete;
  MongooseRenderServiceImpl& operator=(const MongooseRenderServiceImpl&) = delete;

  ~MongooseRenderServiceImpl() override;

  // mongoose::mojom::MongooseRenderService:
  void DrawText(std::vector<mojom::TextDataPtr> data) override;
  void CheckNetworkRequest(mojom::NetworkRequestDataPtr request, CheckNetworkRequestCallback callback) override;
  void DrawImageBitmap(mojom::BitmapDataPtr bitmap) override;
  void SetHybridAsciiMode(bool enabled) override;
  void SetTitle(const std::string& title) override;

 private:
  mojo::Receiver<mojom::MongooseRenderService> receiver_;

  // Handle to the Rust libmongoose orchestrator
  MongooseHandle mongoose_handle_;
};

}  // namespace mongoose

#endif  // MONGOOSE_SRC_BROWSER_RENDER_SERVICE_IMPL_H_