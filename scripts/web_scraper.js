const { chromium } = require("playwright");

// Represents the bounding box
// { x: number, y: number, width: number, height: number }

// Represents flex properties
// { flexDirection: string, padding: string, margin: string, alignItems: string, justifyContent: string }

// Represents an AST Node
// { type: string, rect: Rect, styles: FlexStyles, text?: string, value?: string, children: Node[] }

function extractTopology() {
    function getStyles(el) {
        const computed = window.getComputedStyle(el);
        return {
            flexDirection: computed.flexDirection || "row",
            padding: computed.padding || "0px",
            margin: computed.margin || "0px",
            alignItems: computed.alignItems || "stretch",
            justifyContent: computed.justifyContent || "flex-start",
        };
    }

    function getRect(el) {
        const rect = el.getBoundingClientRect();
        return {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        };
    }

    function isVisible(el) {
        const computed = window.getComputedStyle(el);
        if (computed.display === 'none' || computed.visibility === 'hidden' || computed.opacity === '0') return false;
        const rect = el.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) return false;
        return true;
    }

    function traverse(el) {
        if (!isVisible(el)) return null;

        const tagName = el.tagName.toLowerCase();

        // Skip script, style, meta, etc.
        if (["script", "style", "meta", "link", "noscript", "head", "title"].includes(tagName)) {
            return null;
        }

        let type = "Box";
        if (tagName === "img" || tagName === "svg") type = "Image";
        else if (["input", "textarea", "select"].includes(tagName)) type = "Input";
        else if (["ul", "ol"].includes(tagName)) type = "List";

        let text = "";
        if (el.childNodes.length === 1 && el.childNodes[0].nodeType === Node.TEXT_NODE) {
            const txt = el.childNodes[0].textContent.trim();
            if (txt) {
                type = "Text";
                text = txt;
            }
        }

        let value = "";
        if (type === "Input") {
            value = el.value || el.placeholder || "";
        }

        const children = [];
        for (let i = 0; i < el.children.length; i++) {
            const childNode = traverse(el.children[i]);
            if (childNode) {
                children.push(childNode);
            }
        }

        return {
            type,
            rect: getRect(el),
            styles: getStyles(el),
            ...(text ? { text } : {}),
            ...(value ? { value } : {}),
            children
        };
    }

    return traverse(document.body);
}

async function main() {
    const url = process.argv[2];
    if (!url) {
        console.error("Usage: node scripts/web_scraper.js <url>");
        process.exit(1);
    }

    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();

    try {
        await page.goto(url, { waitUntil: 'networkidle' });
        const ast = await page.evaluate(extractTopology);
        console.log(JSON.stringify(ast, null, 2));
    } catch (error) {
        console.error("Failed to extract AST:", error);
    } finally {
        await browser.close();
    }
}

if (require.main === module) {
    main();
}