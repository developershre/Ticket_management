// HTML5 Drag & Drop Support for TaskFlow (Dioxus Desktop / Linux WebKitGTK)
(function () {
    let activeDragId = null;

    document.addEventListener("dragstart", function (e) {
        var target = e.target;
        var el = target && target.closest ? target.closest('[draggable="true"]') : null;
        if (!el) return;

        var id = el.getAttribute("data-drag-id") || el.getAttribute("id") || "item";
        activeDragId = id;

        if (e.dataTransfer) {
            try {
                e.dataTransfer.effectAllowed = "all";
                e.dataTransfer.setData("text/plain", id);
                e.dataTransfer.setData("application/x-taskflow-id", id);
            } catch (err) {}
        }
    }, false);

    document.addEventListener("dragover", function (e) {
        e.preventDefault();
        if (e.dataTransfer) {
            try {
                e.dataTransfer.dropEffect = "move";
            } catch (err) {}
        }
    }, false);

    document.addEventListener("dragenter", function (e) {
        e.preventDefault();
        if (e.dataTransfer) {
            try {
                e.dataTransfer.dropEffect = "move";
            } catch (err) {}
        }
    }, false);

    document.addEventListener("drop", function (e) {
        if (e.dataTransfer) {
            try {
                e.dataTransfer.dropEffect = "move";
            } catch (err) {}
        }
    }, false);

    document.addEventListener("dragend", function () {
        activeDragId = null;
    }, false);
})();
