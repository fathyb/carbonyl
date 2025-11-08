// Copyright 2025 Mongoose Browser Project
// Use of this source code is governed by an MIT-style license.

#ifndef MONGOOSE_CORE_CONTENT_BROWSER_MONGOOSE_URL_LOADER_INTERCEPTOR_H_
#define MONGOOSE_CORE_CONTENT_BROWSER_MONGOOSE_URL_LOADER_INTERCEPTOR_H_

#include "content/public/browser/url_loader_request_interceptor.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "third_party/blink/public/mojom/loader/resource_load_info.mojom-shared.h"
#include "libmongoose/browser/mongoose.mojom.h"

namespace content {

/// Network request interceptor for Mongoose privacy engine
///
/// This interceptor hooks into Chromium's network stack and calls the Rust
/// privacy engine via Mojo IPC to check if requests should be blocked.
///
/// Integration points:
/// - Called on every network request before URLLoader creation
/// - Makes synchronous Mojo call to Rust (must be fast!)
/// - Returns ERR_BLOCKED_BY_CLIENT for blocked requests
class MongooseURLLoaderInterceptor : public URLLoaderRequestInterceptor {
 public:
  MongooseURLLoaderInterceptor();
  ~MongooseURLLoaderInterceptor() override;

  // Disallow copy and assign
  MongooseURLLoaderInterceptor(const MongooseURLLoaderInterceptor&) = delete;
  MongooseURLLoaderInterceptor& operator=(const MongooseURLLoaderInterceptor&) = delete;

  // URLLoaderRequestInterceptor implementation
  //
  // This is called for every network request before the URLLoader is created.
  // If the privacy engine blocks the request, we return a custom loader that
  // immediately fails with ERR_BLOCKED_BY_CLIENT.
  void MaybeCreateLoader(
      const network::ResourceRequest& tentative_resource_request,
      BrowserContext* browser_context,
      LoaderCallback callback,
      FallbackCallback fallback_callback) override;

  // Set the Mojo remote for testing
  void SetMongooseServiceForTesting(
      mojo::Remote<mongoose::mojom::MongooseRenderService> service);

 private:
  // Check if a request should be blocked
  bool ShouldBlockRequest(const network::ResourceRequest& request,
                         std::string* matched_filter);

  // Mojo interface to Rust privacy engine
  mojo::Remote<mongoose::mojom::MongooseRenderService> mongoose_service_;

  // Statistics (for debugging)
  int total_requests_ = 0;
  int blocked_requests_ = 0;
};

}  // namespace content

#endif  // MONGOOSE_CORE_CONTENT_BROWSER_MONGOOSE_URL_LOADER_INTERCEPTOR_H_
