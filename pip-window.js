const els = {};
let ready = false;
let pendingPayload = null;
let currentPayload = null;

window.__PIP_KANPE_RENDER__ = (payload) => {
  if (!ready) {
    pendingPayload = payload;
    return;
  }

  renderPip(payload);
};

window.addEventListener("DOMContentLoaded", init);

async function init() {
  bindElements();
  bindEvents();
  ready = true;
  if (pendingPayload) {
    renderPip(pendingPayload);
    pendingPayload = null;
  }
  await invokeDesktop("request_pip_snapshot").catch((error) => {
    console.warn("PiP snapshot request failed", error);
  });
}

function bindElements() {
  ["pip-shell", "pip-image", "pip-empty", "pip-controls", "pip-prev", "pip-label", "pip-next", "pip-close"].forEach((id) => {
    els[toCamel(id)] = document.getElementById(id);
  });
}

function bindEvents() {
  els.pipPrev.addEventListener("click", () => stepCard(-1));
  els.pipNext.addEventListener("click", () => stepCard(1));
  els.pipShell.addEventListener("click", handlePipHitAreaClick);
  els.pipClose.addEventListener("click", (event) => {
    // シェルのクリック判定（矢印の当たり判定）に伝播させない。
    event.stopPropagation();
    closePipWindow();
  });
}

async function closePipWindow() {
  await invokeDesktop("close_pip_window").catch((error) => {
    console.warn("PiP close failed", error);
  });
}

async function stepCard(direction) {
  await invokeDesktop("step_pip_card", { direction }).catch((error) => {
    console.warn("PiP navigation failed", error);
  });
}

function renderPip(payload) {
  if (!payload || typeof payload !== "object") {
    showMessage("画像情報を受信できませんでした。");
    return;
  }

  currentPayload = payload;
  document.title = payload.title || payload.strings?.fallbackTitle || "PiP カンペ";
  applyButtonTitles(payload.strings);
  applyControlClasses(payload.controls);
  applyTitleBarState(Boolean(payload.controls?.hideTitleBar));

  const hasCard = Boolean(payload.hasCard && payload.imageSrc);
  els.pipImage.style.display = hasCard ? "block" : "none";
  els.pipEmpty.style.display = hasCard ? "none" : "grid";
  els.pipImage.classList.toggle("cover", payload.fitMode === "cover");
  els.pipShell.classList.toggle("separate", Boolean(payload.controls?.separate));
  els.pipShell.classList.toggle("auto-hide-controls", Boolean(payload.controls?.autoHide));

  if (hasCard) {
    els.pipImage.src = payload.imageSrc;
    els.pipImage.alt = payload.imageAlt || "";
    els.pipLabel.textContent = payload.label || "";
  } else {
    els.pipImage.removeAttribute("src");
    els.pipImage.alt = "";
    els.pipLabel.textContent = "";
    els.pipEmpty.textContent = payload.message || payload.strings?.noVisible || "表示できる画像がありません。";
  }

  const canNavigate = Boolean(payload.canNavigate);
  els.pipPrev.disabled = !canNavigate;
  els.pipNext.disabled = !canNavigate;
  updateButtonLabels(Boolean(payload.controls?.vertical));
}

function applyButtonTitles(strings = {}) {
  if (strings.close) {
    els.pipClose.title = strings.close;
    els.pipClose.setAttribute("aria-label", strings.close);
  }
  if (strings.previousImage) {
    els.pipPrev.title = strings.previousImage;
  }
  if (strings.nextImage) {
    els.pipNext.title = strings.nextImage;
  }
}

function applyControlClasses(controls = {}) {
  const classes = [
    "small",
    "medium",
    "large",
    "horizontal",
    "vertical",
    "vertical-left",
    "vertical-right",
    "full-height-buttons",
    "top",
    "middle",
    "bottom",
    "background-solid",
    "background-translucent",
    "background-clear",
    "separate",
    "label-hidden",
  ];

  els.pipControls.classList.remove(...classes);
  els.pipControls.classList.add(
    controls.size || "medium",
    controls.placement || "horizontal",
    controls.position || "bottom",
    controls.background || "background-solid",
  );
  els.pipControls.classList.toggle("vertical", Boolean(controls.vertical));
  els.pipControls.classList.toggle("full-height-buttons", Boolean(controls.fullHeightButtons));
  els.pipControls.classList.toggle("separate", Boolean(controls.separate));
  els.pipControls.classList.toggle("label-hidden", Boolean(controls.labelHidden));
}

// タイトルバーを消すと OS 側のドラッグ領域が無くなるため、
// 画像・空表示部分を data-tauri-drag-region にして小窓を動かせるようにする。
function applyTitleBarState(hidden) {
  els.pipShell.classList.toggle("title-bar-hidden", hidden);
  [els.pipImage, els.pipEmpty].forEach((el) => {
    if (hidden) {
      el.setAttribute("data-tauri-drag-region", "");
    } else {
      el.removeAttribute("data-tauri-drag-region");
    }
  });
}

function updateButtonLabels(vertical) {
  els.pipPrev.textContent = vertical ? "↑" : "←";
  els.pipNext.textContent = vertical ? "↓" : "→";
}

// 「矢印の当たり判定を縦いっぱいにする」用のクリック判定。
// 見た目のボタンは小さいまま、矢印と同じ縦ラインを小窓全体の操作領域として扱う。
function handlePipHitAreaClick(event) {
  const controls = currentPayload?.controls;
  if (!controls?.fullHeightButtons || !currentPayload?.canNavigate) {
    return;
  }

  const target = event.target;
  if (target && typeof target.closest === "function" && target.closest(".pip-button")) {
    return;
  }

  const shellRect = els.pipShell.getBoundingClientRect();
  const previousRect = els.pipPrev.getBoundingClientRect();
  const nextRect = els.pipNext.getBoundingClientRect();
  if (
    shellRect.width <= 0 ||
    shellRect.height <= 0 ||
    previousRect.width <= 0 ||
    nextRect.width <= 0
  ) {
    return;
  }

  if (controls.vertical) {
    const laneLeft = Math.min(previousRect.left, nextRect.left);
    const laneRight = Math.max(previousRect.right, nextRect.right);
    if (event.clientX < laneLeft || event.clientX > laneRight) {
      return;
    }

    event.preventDefault();
    stepCard(event.clientY < shellRect.top + shellRect.height / 2 ? -1 : 1);
    return;
  }

  if (event.clientX >= previousRect.left && event.clientX <= previousRect.right) {
    event.preventDefault();
    stepCard(-1);
  } else if (event.clientX >= nextRect.left && event.clientX <= nextRect.right) {
    event.preventDefault();
    stepCard(1);
  }
}

function showMessage(message) {
  currentPayload = null;
  els.pipImage.style.display = "none";
  els.pipImage.removeAttribute("src");
  els.pipEmpty.style.display = "grid";
  els.pipEmpty.textContent = message;
  els.pipPrev.disabled = true;
  els.pipNext.disabled = true;
}

function invokeDesktop(command, payload = {}) {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return Promise.reject(new Error("Tauri API is unavailable"));
  }
  return invoke(command, payload);
}

function toCamel(value) {
  return value.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}
