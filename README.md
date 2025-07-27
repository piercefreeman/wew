<!--lint disable no-literal-urls-->
<div align="center">
  <h1>WEW</h1>
</div>
<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/mycrl/wew/check.yml?branch=main&style=flat-square"/>
  <img src="https://img.shields.io/crates/v/wew?style=flat-square"/>
  <img src="https://img.shields.io/docsrs/wew?style=flat-square"/>
  <img src="https://img.shields.io/github/license/mycrl/wew?style=flat-square"/>
  <img src="https://img.shields.io/github/issues/mycrl/wew?style=flat-square"/>
  <img src="https://img.shields.io/github/stars/mycrl/wew?style=flat-square"/>
</div>
<div align="center">
  <sup>
    current cef version: 
    <a href="https://cef-builds.spotifycdn.com/index.html">137.0.17+gf354b0e+chromium-137.0.7151.104</a>
  </sup>
  </br>
  <sup>platform supported: windows / macOS / linux (x11)</sup>
</div>

---

Wew is a cross-platform WebView rendering library based on [Chromium Embedded Framework (CEF)](https://github.com/chromiumembedded/cef). It supports mouse, keyboard, touch, input methods, off-screen rendering, and communication with web pages.

## Process Model

> [!NOTE]  
> This project is based on CEF, so the process model is the same as CEF and Chromium. Chromium and CEF (Chromium Embedded Framework) have a very modular process model design, adopting a multi-process architecture primarily to enhance security, stability, and performance. The overall architecture is a multi-process model (Multi-Process Architecture).

#### Browser Process

The main process responsible for managing the entire Chromium/CEF lifecycle. It handles window management, user input (events) processing, network requests (through network services or proxy network processes), scheduling Renderer processes, security policy control, and communication with the system.

#### Renderer Process

Responsible for web page parsing, layout, rendering, JavaScript execution, etc. It executes HTML, CSS, JavaScript, integrates closely with the Blink engine, and each tab/iframe typically corresponds to an independent rendering process (more granular after Site Isolation).

#### GPU Process

Used for hardware-accelerated graphics rendering. It handles WebGL, Canvas, CSS accelerated rendering, etc., and communicates with the operating system's GPU drivers.

#### Utility Processes

Run specific tasks unrelated to web page rendering. Network Service: responsible for all network requests, Audio/Video decode service, database storage, extension processes, etc.

#### Zygote Process (Linux only)

Used for quickly spawning renderer processes to improve performance.

---

Inter-process communication (IPC) is used to synchronize state between Browser and Renderer, implement security controls and permission passing, and transmit events (such as clicks, scrolling, JavaScript calls). Security isolation is ensured by running Renderers in sandboxes, with each website/domain potentially running in an independent Renderer process, and high-privilege operations like GPU and network access not exposed to rendering processes.

## Thread Considerations

In the current project, WebView and Runtime calls are best executed on the UI thread, which is the main thread of the application process.

Creating a Runtime must be completed on the UI thread, and all message loop calls must also be operated on the UI thread.

Other calls should be executed on the UI thread whenever possible, unless it is truly unavoidable. Although these calls can run on any thread, there is currently no guarantee that they will not cause other side effects.

However, it is important to note that if the WebView manages window events on its own, such as not using off-screen rendering, then the WebView can be created on any thread.

## Packaging and Running

> [!NOTE]  
> Please note that due to CEF's packaging method, it cannot be integrated with Cargo. The CEF runtime requires many resource files and executables to be placed together, and on macOS, it also needs to follow strict and specific packaging requirements that Cargo cannot handle. Therefore, using it currently requires extensive manual operations or writing custom scripts to automate the process. You can refer to the ([Windowless Rendering](./examples/windowless_rendering)) example to understand how to package your application.

As you might see from the size of the rust library (~2mb), we're not actually including Chromium within our build project. It doesn't make sense to - because any downstream executable is going to have to update its own linking flags after the project's build to point to the right location of the runtime Chromium framework files anyway.

To help make this easier, we include a wrapper around `cargo build` that will compile your wew project and link it to the right version of Chromium.

```bash
cargo build --release # make sure the main lib is built
./target/release/wrap_wew --entrypoint ./examples/main_thread --release
```

The output of this pipeline is a local .tar file. Unzipping the tar file will reveal your executables, which you should be able to exec as normal:

```bash
tar -xf main_thread.tar
open main_thread/main_thread.app
```

## Communication with Web Pages

This library's runtime will inject a global object into web pages for communication between Rust and web pages.

```typescript
declare global {
    interface Window {
        MessageTransport: {
            on: (handle: (message: string) => void) => void;
            send: (message: string) => void;
        };
    }
}
```

Usage example:

```typescript
window.MessageTransport.on((message: string) => {
    console.log("Received message from Rust:", message);
});

window.MessageTransport.send("Send message to Rust");
```

`WebViewHandler::on_message` is used to receive messages sent by `MessageTransport.send`, while `MessageTransport.on` is used to receive messages sent by `WebView::send_message`. Sending and receiving messages are full-duplex and asynchronous.

## License

[MIT](./LICENSE) Copyright (c) 2025 Mr.Panda.
