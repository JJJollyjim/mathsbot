# mathsbot
> Discord bot to render mathematics from messages using LaTeX.

![screenshot of example usage in Discord](https://user-images.githubusercontent.com/691552/31049854-352a2836-a699-11e7-8b7e-e3ba9121ce98.png)

Currently, it picks up maths between single-dollar-sign delimiters or in `\begin{}...\end{}` environments, and is not capable of displaying non-inline LaTeX blocks.

The rendering process pipes the message through PDFLaTeX using the [`standalone`](https://ctan.org/pkg/standalone?lang=en) document class, then gets a `png` file with ImageMagick, which is uploaded to Discord.

The bot also supports updating edited messages.
