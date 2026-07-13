import React from "react";

const Info = () => {
  return (
    <div className="Info">
      <h1>Sandspiel</h1>
      <p>
        作者：<a href="https://maxbittker.com">max bittker</a>
      </p>
      <hr />
      <br />
      <p>
        欢迎来玩！希望你在这个小小的物理沙盒里探索材料、制造反应，也获得一点放松。
      </p>
      <p>
        这类“落沙游戏”曾带来很多想象力和乐趣。Sandspiel 的主要灵感来自 ha55ii 的{" "}
        <a href="https://dan-ball.jp/en/javagame/dust/">Powder Game</a>。
      </p>
      <br />
      <p>
        想了解灵感、架构和制作历史，可以阅读作者的技术博客：{" "}
        <a href="https://maxbittker.com/making-sandspiel">Making Sandspiel</a>
      </p>
      <br />
      <p>
        你也可以查看{" "}
        <a href="https://github.com/maxbittker/sandspiel">源代码</a>，或在 GitHub{" "}
        <a href="https://github.com/maxbittker/sandspiel/issues">反馈问题</a>。
      </p>
      <br />
      <p>
        请友善创作和分享作品。这里应该是一个没有欺凌、种族歧视、跨性别恐惧、同性恋恐惧或其他偏见的游玩空间。
        如果有什么不对，欢迎联系作者：{" "}
        <a href="mailto:maxbittker@gmail.com">maxbittker@gmail.com</a> 或{" "}
        <a href="https://twitter.com/maxbittker">Twitter @maxbittker</a>。
      </p>
      <br />
      <hr />
      <br />
      <h2>元素说明：</h2>
      <h4>墙</h4>
      不可破坏。
      <h4>沙子</h4>
      会沉入水中。
      <h4>水</h4>
      可以灭火。
      <h4>石头</h4>
      会形成拱形结构，在压力下会变成沙子。
      <h4>冰</h4>
      会冻结水，而且很滑。
      <h4>气体</h4>
      非常易燃。
      <h4>复制器</h4>
      会复制它接触到的第一个元素。
      <h4>螨虫</h4>
      会吃木头和植物，但喜欢尘粉；在冰上会滑动。
      <h4>木头</h4>
      结实，但会被自然分解。
      <h4>植物</h4>
      在潮湿环境中生长得更好。
      <h4>真菌</h4>
      会在各种材料上蔓延。
      <h4>种子</h4>
      会在沙子、植物和真菌上生长。
      <h4>火</h4>
      很热。
      <h4>岩浆</h4>
      易燃，而且很重。
      <h4>酸液</h4>
      会腐蚀其他元素。
      <h4>尘粉</h4>
      很漂亮，但有爆炸风险。
      <h4>油</h4>
      被点燃时会产生烟雾。
      <h4>火箭</h4>
      会爆炸成它接触到的第一个元素的副本。
      <h4>橡皮</h4>
      用来擦除。
      <hr />
    </div>
  );
};

export default Info;
