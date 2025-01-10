'use strict';


const DOWN_KEY        = 'ArrowDown';
const LEFT_KEY        = 'ArrowLeft';
const RIGHT_KEY       = 'ArrowRight';
const CLICK_VIDEO_KEY = ' ';
const UP_KEY          = 'ArrowUp';


const DOWN_DIRECTION  = 0;
const LEFT_DIRECTION  = 1;
const RIGHT_DIRECTION = 2;
const UP_DIRECTION    = 3;


/**
 * Get the current amount of video links per row
 * @param {Element} element
 */
function get_column_count(element)
{
	let parent = element.parentNode;
	if (!parent)
		return NaN;
	return parseInt(getComputedStyle(parent).getPropertyValue('--columns'));
}


/**
 * Handle keys as navigation shortcuts
 * @param {KeyboardEvent} event
 */
function handle_key(event)
{
	if (event.altKey || event.ctrlKey || event.shiftKey)
		return;

	switch (event.key) {
		case DOWN_KEY:
			event.preventDefault();
			move_in_direction(DOWN_DIRECTION);
			return;
		case LEFT_KEY:
			event.preventDefault();
			move_in_direction(LEFT_DIRECTION);
			return;
		case RIGHT_KEY:
			event.preventDefault();
			move_in_direction(RIGHT_DIRECTION);
			return;
		case CLICK_VIDEO_KEY:
			const focused = document.activeElement;
			if (!focused || focused.tagName !== 'A')
				return;
			event.preventDefault();
			focused.click();
			return;
		case UP_KEY:
			event.preventDefault();
			move_in_direction(UP_DIRECTION);
			return;
		default:
			move_to_letter(event);
			return;
	}
}


/**
 * Move up, down, left, or right
 * @param {number} direction
 */
function move_in_direction(direction)
{
	const focused = document.activeElement;

	// If not focused on a link, focus on the first link
	if (!focused || focused.tagName !== 'A') {
		const target = document.querySelector('.video_link');
		if (target)
			target.focus();
		return;
	}

	// Get the column count until the target link
	let columns;
	switch (direction) {
		case DOWN_DIRECTION: {
			columns = get_column_count(focused);
			break;
		}
		case LEFT_DIRECTION:
			columns = -1;
			break;
		case RIGHT_DIRECTION: {
			columns = 1;
			break;
		}
		case UP_DIRECTION: {
			columns = -get_column_count(focused);
			break;
		}
		default:
			return;
	}
	if (!columns)
		return;

	let parent = focused.parentNode;
	if (!parent)
		return;
	if (columns > 0) {
		for (let i = 0; i != columns; i++) {
			const new_parent = parent.nextSibling;
			if (!new_parent)
				break;
			parent = new_parent;
		}
	}
	else {
		for (let i = 0; i != columns; i--) {
			const new_parent = parent.previousSibling;
			if (!new_parent)
				break;
			parent = new_parent;
		}
	}

	if (parent && parent.children.length)
		parent.children[0].focus();
}


/**
 * Skip until the video link starting with the typed key
 * @param {KeyboardEvent} event
 */
function move_to_letter(event)
{
	// Get a single letter or stop
	const key = event.key.toLocaleLowerCase();
	if (key.length !== 1)
		return;

	// Focus on the first video link with that letter
	for (const link of document.getElementsByClassName('video_link')) {
		for (const child of link.children) {
			const title = child.innerText[0];
			if (typeof(title) !== 'string' || title.toLocaleLowerCase() !== key)
				continue;
			event.preventDefault();
			link.focus();
			return;
		}
	}
}


document.onkeydown = handle_key;
