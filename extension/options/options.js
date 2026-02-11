const input = document.getElementById("daemonUrl");
const message = document.getElementById("message");

chrome.storage.local.get(["daemonUrl"], ({ daemonUrl }) => {
  input.value = daemonUrl || "http://127.0.0.1:8472";
});

document.getElementById("save").addEventListener("click", async () => {
  await chrome.storage.local.set({ daemonUrl: input.value.trim() });
  message.textContent = "Saved";
});
