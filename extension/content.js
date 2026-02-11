// Minimal capture: Ctrl/Cmd + click on links ending with common archive/media extensions.
document.addEventListener("click", (event) => {
  const link = event.target.closest("a[href]");
  if (!link) return;

  const interesting = /\.(zip|7z|rar|tar|gz|mp4|mkv|iso|exe|msi)$/i.test(link.href);
  const hotkey = event.ctrlKey || event.metaKey;
  if (!interesting || !hotkey) return;

  chrome.runtime.sendMessage({
    type: "NETDOUT_QUEUE",
    url: link.href,
    output: "./download-from-browser.bin"
  });
});
