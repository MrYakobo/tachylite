const headings = document.querySelectorAll('#desktop article h1, #desktop article h2, #desktop article h3, #desktop article h4, #desktop article h5, #desktop article h6')
const tocLinks = document.querySelectorAll('#desktop .toc a')

const container = document.querySelector("#desktop article")

function is_scrolled_by(container, el, margin) {
    return (container.scrollTop > (el.offsetTop + el.offsetHeight - margin))
}

function scroll_handler() {
    // reset all to inactive
    for (const link of tocLinks) {
        link.classList.remove('highlighted')
    }

    // iterate backwards, and break on first match
    for (let i = headings.length - 1; i >= 0; i--) {
        const heading = headings[i]
        const associated_toclink = Array.from(tocLinks).find(tocLink => {
            const href = tocLink.getAttribute('href')
            const heading_id = heading.querySelector("a") || false
            if (!heading_id)
                return false

            return href.split("#")[1] == heading_id.href.split("#")[1]
        })

        if (is_scrolled_by(container, heading, 100)) {
            console.log('highlighting', heading.textContent, associated_toclink)
            if (associated_toclink)
                associated_toclink.classList.add('highlighted')

            break
        }
    }

}

container.addEventListener('scroll', () => {
    scroll_handler()
});

