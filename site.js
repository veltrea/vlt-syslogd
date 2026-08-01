// Veltrea Syslog Server — サイトの動き。
// ビルド工程を持たないので、ここに置くのは「無くても読める」ものだけに限る。
(function () {
  "use strict";

  /* --- コマンド欄のコピー ------------------------------------------------ */
  document.querySelectorAll("pre.cmd").forEach(function (pre) {
    if (!navigator.clipboard) return;

    var btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = "コピー";
    var label = pre.closest("[lang='en']") ? "en" : document.documentElement.lang;
    var words =
      label === "en"
        ? { idle: "Copy", done: "Copied", fail: "Press ⌘C" }
        : { idle: "コピー", done: "コピーしました", fail: "手でコピーしてください" };
    btn.textContent = words.idle;
    btn.setAttribute("aria-label", words.idle);

    btn.addEventListener("click", function () {
      var code = pre.querySelector("code");
      navigator.clipboard.writeText((code || pre).innerText.trim()).then(
        function () {
          btn.textContent = words.done;
          setTimeout(function () {
            btn.textContent = words.idle;
          }, 1600);
        },
        function () {
          btn.textContent = words.fail;
        }
      );
    });

    pre.appendChild(btn);
  });

  /* --- マニュアルの目次: いま読んでいる節を示す ------------------------- */
  var toc = document.querySelector(".toc");
  if (!toc || !("IntersectionObserver" in window)) return;

  var links = new Map();
  toc.querySelectorAll('a[href^="#"]').forEach(function (a) {
    var el = document.getElementById(decodeURIComponent(a.hash.slice(1)));
    if (el) links.set(el, a);
  });
  if (!links.size) return;

  var current = null;
  var observer = new IntersectionObserver(
    function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        var a = links.get(entry.target);
        if (!a || a === current) return;
        if (current) current.classList.remove("is-current");
        a.classList.add("is-current");
        current = a;
      });
    },
    { rootMargin: "-15% 0px -70% 0px", threshold: 0 }
  );

  links.forEach(function (_a, el) {
    observer.observe(el);
  });
})();
