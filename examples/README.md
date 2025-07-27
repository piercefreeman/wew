## Examples

- [main-thread](./main_thread/) - A simple example that lets wew manage the message loop and create native windows on its own.
- [windowless-rendering](./windowless_rendering/) - This example demonstrates how to use the off-screen rendering mode to create a webview and control the rendering of webview output yourself. The application's main thread event loop is managed by winit, and the application window is also created by winit. The webview doesn't create any windows. This project forwards winit window events to the webview and is responsible for driving the webview's message pump, while the video frames rendered by the webview are rendered to the winit-created window through wgpu.

---

Since this project is based on CEF and requires specific packaging methods to standardize the project structure, we use Node.js scripts to automate the packaging process.

You need to package this example project first:

```bash
./target/debug/wrap_wew --entrypoint ./examples/main_thread --release
```

After compilation, the build artifacts are in the `target/examples` directory. On macOS, you'll see a `.app` file; on Windows or Linux, you'll see a folder. The `.app` file can be run directly, while for folders, the executable file is located inside the folder.
