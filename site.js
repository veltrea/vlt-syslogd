// Veltrea Syslog Server — サイトの動き。
// ビルド工程を持たないので、ここに置くのは「無くても読める」ものだけに限る。
(function () {
  "use strict";

  var reduced =
    window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  /* --- コマンド欄のコピー ------------------------------------------------ */
  document.querySelectorAll("pre.cmd").forEach(function (pre) {
    if (!navigator.clipboard) return;

    var en = document.documentElement.lang === "en";
    var words = en
      ? { idle: "COPY", done: "COPIED", fail: "USE ⌘C" }
      : { idle: "コピー", done: "コピーしました", fail: "手でコピーを" };

    var btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = words.idle;

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

  /* --- 生きているログ ---------------------------------------------------- */
  // 一面の卓に流れる行。中身は実機の撮影に使ったものと同じ demo feed で、
  // 「1 通ごとに符号化を判定する」という動きをその場で見せるためだけにある。
  var stream = document.querySelector("[data-stream]");
  var feedEl = document.getElementById("feed-data");

  if (stream && feedEl) {
    var feed = JSON.parse(feedEl.textContent);
    var rows = stream.querySelector(".stream__rows");
    var counter = document.querySelector("[data-count]");
    var MAX = 16;
    var i = 0;
    var total = 0;
    var timer = null;

    var pad = function (n, w) {
      return String(n).padStart(w, "0");
    };

    var stamp = function () {
      var d = new Date();
      return (
        pad(d.getHours(), 2) +
        ":" +
        pad(d.getMinutes(), 2) +
        ":" +
        pad(d.getSeconds(), 2) +
        "." +
        pad(d.getMilliseconds(), 3)
      );
    };

    var add = function (animate) {
      var f = feed[i % feed.length];
      i += 1;
      total += 1;

      var row = document.createElement("div");
      row.className = "stream__row is-" + f.k + (animate ? " is-new" : "");
      row.innerHTML =
        '<span class="t"></span><span class="g"></span>' +
        '<span class="e"></span><span class="m"></span>';
      row.children[0].textContent = stamp();
      row.children[1].textContent = f.tag;
      row.children[2].textContent = f.enc;
      row.children[3].textContent = f.msg;
      row.children[3].classList.add("s");

      rows.appendChild(row);
      while (rows.children.length > MAX) rows.removeChild(rows.firstChild);
      if (counter) counter.textContent = String(total).padStart(4, "0");
    };

    // 最初から埋まった状態で見せる（空の卓を見せない）
    for (var n = 0; n < MAX; n++) add(false);

    if (!reduced) {
      var start = function () {
        if (timer) return;
        timer = setInterval(function () {
          add(true);
        }, 1100);
      };
      var stop = function () {
        clearInterval(timer);
        timer = null;
      };

      if ("IntersectionObserver" in window) {
        new IntersectionObserver(
          function (entries) {
            entries.forEach(function (e) {
              if (e.isIntersecting) start();
              else stop();
            });
          },
          { threshold: 0 }
        ).observe(stream);
      } else {
        start();
      }

      document.addEventListener("visibilitychange", function () {
        if (document.hidden) stop();
      });
    }
  }

  /* --- 走査線の行程を卓の高さに合わせる ---------------------------------- */
  var console_ = document.querySelector(".console");
  if (console_ && !reduced) {
    var setSweep = function () {
      console_.style.setProperty("--sweep", console_.offsetHeight + "px");
    };
    setSweep();
    window.addEventListener("resize", setSweep);
  }

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
