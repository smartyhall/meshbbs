//! Unix fortune cookie mini-feature used by public channel command <prefix>FORTUNE (default prefix `^`).
//!
//! This module provides a stateless fortune cookie system inspired by the classic Unix
//! `fortune` command. It contains a curated database of 400 mixed fortunes: witty,
//! funny, thoughtful, and clean—each under 200 characters.
//! humor, philosophical insights, and motivational messages.
//!
//! # Behavior
//!
//! - **Stateless**: No persistence required; pure function calls
//! - **Delivery**: Public broadcast only (best-effort), same reliability posture as `<prefix>SLOT`
//! - **Rate limiting**: Handled by `PublicState.allow_fortune` (5-second per-node cooldown)
//! - **Mesh-optimized**: All entries under 200 characters for efficient transmission
//!
//! # Fortune Database
//!
//! The fortune database is compiled from public domain sources including:
//! - Classic BSD Unix fortune collections
//! - Programming and technology wisdom
//! - Literature and philosophical quotes
//! - Clean, family-friendly humor
//! - Motivational and inspirational content
//!
//! # Usage
//!
//! Users send `<prefix>FORTUNE` on the public channel to receive a random fortune (default `^FORTUNE`):
//!
//! ```text
//! User: <prefix>FORTUNE
//! BBS:  <prefix>FORTUNE ⟶ The only true wisdom is in knowing you know nothing. — Socrates
//! ```
//!
//! # Thread Safety
//!
//! All functions in this module are thread-safe and can be called concurrently
//! from multiple tasks without synchronization.

use rand::Rng;

/// Curated collection of fortune cookies from classic Unix databases.
/// Mix of wisdom, literature, programming quotes, and clean humor.
/// All entries under 200 characters for mesh network compatibility.
const FORTUNES: [&str; 573] = [
    // Classic wisdom
    "The only true wisdom is in knowing you know nothing. — Socrates",
    "In the middle of difficulty lies opportunity. — Albert Einstein",
    "It is during our darkest moments that we must focus to see the light. — Aristotle",
    "The journey of a thousand miles begins with one step. — Lao Tzu",
    "Yesterday is history, tomorrow is a mystery, today is a gift. — Eleanor Roosevelt",
    "Be yourself; everyone else is already taken. — Oscar Wilde",
    "Two things are infinite: the universe and human stupidity; I'm not sure about the universe. — Einstein",
    "The only way to do great work is to love what you do. — Steve Jobs",
    "Life is what happens when you're busy making other plans. — John Lennon",
    "The future belongs to those who believe in the beauty of their dreams. — Eleanor Roosevelt",
    
    // Programming & Tech
    "There are only two hard things in Computer Science: cache invalidation and naming things. — Phil Karlton",
    "The best way to get a project done faster is to start sooner. — Jim Highsmith",
    "Code is like humor. When you have to explain it, it's bad. — Cory House",
    "First, solve the problem. Then, write the code. — John Johnson",
    "Any fool can write code that a computer can understand. Good programmers write code humans understand. — Martin Fowler",
    "The most important property of a program is whether it accomplishes the intention of its user. — C.A.R. Hoare",
    "Programs must be written for people to read, and only incidentally for machines to execute. — Abelson & Sussman",
    "The best error message is the one that never shows up. — Thomas Fuchs",
    "Debugging is twice as hard as writing the code in the first place. — Brian Kernighan",
    "Talk is cheap. Show me the code. — Linus Torvalds",
    "The computer was born to solve problems that did not exist before. — Bill Gates",
    "Software is a great combination between artistry and engineering. — Bill Gates",
    "It's not a bug – it's an undocumented feature. — Anonymous",
    "There's nothing more permanent than a temporary hack. — Kyle Simpson",
    "Good code is its own best documentation. — Steve McConnell",
    
    // Science & Discovery
    "Science is a way of thinking much more than it is a body of knowledge. — Carl Sagan",
    "The important thing is not to stop questioning. — Albert Einstein",
    "Somewhere, something incredible is waiting to be known. — Carl Sagan",
    "The greatest enemy of knowledge is not ignorance, it is the illusion of knowledge. — Stephen Hawking",
    "What we know is a drop, what we don't know is an ocean. — Isaac Newton",
    "I have not failed. I've just found 10,000 ways that won't work. — Thomas Edison",
    "Research is what I'm doing when I don't know what I'm doing. — Wernher von Braun",
    "The good thing about science is that it's true whether or not you believe in it. — Neil deGrasse Tyson",
    
    // Literature & Philosophy  
    "Not all those who wander are lost. — J.R.R. Tolkien",
    "It was the best of times, it was the worst of times. — Charles Dickens",
    "To be or not to be, that is the question. — William Shakespeare",
    "All that is gold does not glitter. — J.R.R. Tolkien",
    "The pen is mightier than the sword. — Edward Bulwer-Lytton",
    "I think, therefore I am. — René Descartes",
    "The unexamined life is not worth living. — Socrates",
    "Man is condemned to be free. — Jean-Paul Sartre",
    "Hell is other people. — Jean-Paul Sartre",
    "The only constant in life is change. — Heraclitus",
    
    // Clean Humor & Wit
    "I'm not arguing, I'm just explaining why I'm right. — Anonymous",
    "The early bird might get the worm, but the second mouse gets the cheese. — Anonymous",
    "If at first you don't succeed, then skydiving definitely isn't for you. — Steven Wright",
    "I told my wife she was drawing her eyebrows too high. She looked surprised. — Anonymous",
    "Why don't scientists trust atoms? Because they make up everything! — Anonymous",
    "Parallel lines have so much in common. It's a shame they'll never meet. — Anonymous",
    "I'm reading a book about anti-gravity. It's impossible to put down! — Anonymous",
    "Did you hear about the mathematician who's afraid of negative numbers? He stops at nothing! — Anonymous",
    
    // Motivational
    "Success is not final, failure is not fatal: it is the courage to continue that counts. — Churchill",
    "The only impossible journey is the one you never begin. — Tony Robbins",
    "Your limitation—it's only your imagination. — Anonymous",
    "Push yourself, because no one else is going to do it for you. — Anonymous",
    "Great things never come from comfort zones. — Anonymous",
    "Dream it. Wish it. Do it. — Anonymous",
    "Success doesn't just find you. You have to go out and get it. — Anonymous",
    "The harder you work for something, the greater you'll feel when you achieve it. — Anonymous",
    "Don't stop when you're tired. Stop when you're done. — Anonymous",
    "Wake up with determination. Go to bed with satisfaction. — Anonymous",
    
    // Technology & Future
    "The advance of technology is based on making it fit in so you don't really notice it. — Bill Gates",
    "Technology is best when it brings people together. — Matt Mullenweg",
    "The Internet is becoming the town square for the global village of tomorrow. — Bill Gates",
    "Innovation distinguishes between a leader and a follower. — Steve Jobs",
    "The real problem is not whether machines think but whether men do. — B.F. Skinner",
    "Any sufficiently advanced technology is indistinguishable from magic. — Arthur C. Clarke",
    "The future is not some place we are going, but one we are creating. — John Schaar",
    
    // Unix/Computing Culture
    "Unix is simple. It just takes a genius to understand its simplicity. — Dennis Ritchie",
    "UNIX is basically a simple operating system, but you have to be a genius to understand the simplicity. — Dennis Ritchie",
    "The power of Unix lies in the philosophy behind it. — Anonymous",
    "Everything is a file. — Unix Philosophy",
    "Write programs that do one thing and do it well. — Unix Philosophy",
    "Worse is better. — Richard Gabriel",
    "When in doubt, use brute force. — Ken Thompson",
    
    // Classic Sayings
    "A penny saved is a penny earned. — Benjamin Franklin",
    "Actions speak louder than words. — Abraham Lincoln",
    "Fortune favors the bold. — Latin Proverb",
    "Knowledge is power. — Francis Bacon",
    "Time is money. — Benjamin Franklin",
    "Practice makes perfect. — Ancient Proverb",
    "Where there's a will, there's a way. — English Proverb",
    "You can't judge a book by its cover. — English Proverb",
    "The squeaky wheel gets the grease. — American Proverb",
    "Better late than never. — English Proverb",
    "Don't count your chickens before they hatch. — Aesop",
    "Every cloud has a silver lining. — English Proverb",
    "Rome wasn't built in a day. — Medieval French Proverb",
    "When life gives you lemons, make lemonade. — Elbert Hubbard",
    "The grass is always greener on the other side. — English Proverb",
    
    // Math & Logic
    "Mathematics is the language with which God has written the universe. — Galileo",
    "In mathematics you don't understand things. You just get used to them. — John von Neumann",
    "Pure mathematics is, in its way, the poetry of logical ideas. — Albert Einstein",
    "God does not play dice with the universe. — Albert Einstein",
    "Mathematics is not about numbers, equations, computations, or algorithms: it is about understanding. — William Paul Thurston",
    
    // Short Observations
    "Reality is that which, when you stop believing in it, doesn't go away. — Philip K. Dick",
    "The only way to make sense out of change is to plunge into it, move with it, and join the dance. — Alan Watts",
    "We are what we repeatedly do. Excellence, then, is not an act, but a habit. — Aristotle",
    "The best time to plant a tree was 20 years ago. The second best time is now. — Chinese Proverb",
    "A goal without a plan is just a wish. — Antoine de Saint-Exupéry",
    "You miss 100% of the shots you don't take. — Wayne Gretzky",
    "Whether you think you can or you think you can't, you're right. — Henry Ford",
    "It does not matter how slowly you go as long as you do not stop. — Confucius",
    "Everything you've ever wanted is on the other side of fear. — George Addair",
    "Believe you can and you're halfway there. — Theodore Roosevelt",
    "The only person you are destined to become is the person you decide to be. — Ralph Waldo Emerson",
    "Go confidently in the direction of your dreams. Live the life you have imagined. — Henry David Thoreau",
    "Few things can help an individual more than to place responsibility on him. — Booker T. Washington",
    "It is never too late to be what you might have been. — George Eliot",
    "Life is 10% what happens to you and 90% how you react to it. — Charles R. Swindoll",
    
    // Technology Humor
    "There are 10 types of people in the world: those who understand binary and those who don't. — Anonymous",
    "To understand recursion, you must first understand recursion. — Anonymous",
    "It works on my machine. — Every Developer Ever",
    "Have you tried turning it off and on again? — IT Support Everywhere",
    "99 little bugs in the code, 99 little bugs. Take one down, patch it around, 117 little bugs in the code. — Anonymous",
    "Patience is the quiet partner of wisdom. — Anonymous",
    "If debugging is the process of removing bugs, then programming must be the process of putting them in. — Edsger Dijkstra",
    "Measuring programming progress by lines of code is like measuring aircraft building progress by weight. — Bill Gates",
    "The best thing about a boolean is even if you are wrong, you are only off by a bit. — Anonymous",
    "A user interface is like a joke. If you have to explain it, it's not that good. — Martin LeBlanc",
    
    // Final Wisdom
    "The only true failure is the failure to try. — Anonymous",
    "Don't wait for opportunity. Create it. — Anonymous",
    "Life begins at the end of your comfort zone. — Neale Donald Walsch",
    "The difference between ordinary and extraordinary is that little extra. — Jimmy Johnson",
    "Champions don't show up to get everything they want; they show up to give everything they have. — Anonymous",
    "Success is walking from failure to failure with no loss of enthusiasm. — Winston Churchill",
    "The expert in anything was once a beginner. — Helen Hayes",
    "Don't let yesterday take up too much of today. — Will Rogers",
    "You learn more from failure than from success. Don't let it stop you. Failure builds character. — Anonymous",
    "If you are not willing to risk the usual, you will have to settle for the ordinary. — Jim Rohn",
    "Take up one idea. Make that one idea your life. Think of it, dream of it, live on that idea. — Swami Vivekananda",
    "All our dreams can come true if we have the courage to pursue them. — Walt Disney",
    "Good things come to people who wait, but better things come to those who go out and get them. — Anonymous",
    "If you do what you always did, you will get what you always got. — Anonymous",
    "Happiness is not something readymade. It comes from your own actions. — Dalai Lama",
    "The way to get started is to quit talking and begin doing. — Walt Disney",
    "Don't let the fear of losing be greater than the excitement of winning. — Robert Kiyosaki",
    "If you want to lift yourself up, lift up someone else. — Booker T. Washington",
    "Success is not how high you have climbed, but how you make a positive difference. — Roy T. Bennett",
    "What lies behind us and what lies before us are tiny matters compared to what lies within us. — Ralph Waldo Emerson",
    
    // --- Additional originals (clean, witty, thoughtful; all < 200 chars) ---
    "A good plan today beats a perfect plan tomorrow.",
    "If it takes five minutes now, it saves an hour later.",
    "You can't control the wind, but you can trim the sails.",
    "Tiny habits compound like interest.",
    "Be the person your future self thanks.",
    "Momentum loves small wins.",
    "A tidy inbox is a mirage; set filters, not expectations.",
    "If it's not on your calendar, it's on your mind.",
    "Read more books than you buy.",
    "Quality is kindness to your future teammates.",
    "Write letters you’d be proud to receive.",
    "A failing test is a friend with bad news.",
    "Assumptions are invisible walls.",
    "Measure twice, ship once.",
    "Waiting for perfect conditions is a slow way to never start.",
    "If the task scares you, timebox it for ten minutes.",
    "Silence is a superpower in meetings.",
    "Make the right thing the easy thing.",
    "Don't raise your voice; improve your argument.",
    "Urgent often means someone else's poor planning.",
    "Be brave enough to be bad at something new.",
    "Curiosity outlives certainty.",
    "Complexity is a debt paid in panics.",
    "Decisions are data with a deadline.",
    "A sketch is a conversation with reality.",
    "A good journal is time travel for the soul.",
    "If you can't explain it simply, you don't understand it well enough.",
    "Sleep is the cheapest performance upgrade.",
    "Shortcuts are long when they skip learning.",
    "Every 'just this once' teaches a habit.",
    "Meetings without notes are rumors with chairs.",
    "Tools don't change you; habits do.",
    "If everything is a priority, nothing is.",
    "Kindness scales.",
    "You can't pour from an empty battery.",
    "Slow is smooth; smooth is fast.",
    "Less friction, more flow.",
    "The best feature is the one you can remove.",
    "Refactor before it begs for forgiveness.",
    "Automate the boring, not the broken.",
    "Your calendar reveals your values.",
    "Outcomes over outputs.",
    "Feedback is fuel. Request it early.",
    "If you need a meeting, propose an agenda.",
    "Process is a promise you make to your future.",
    "A checklist is humility in bullet points.",
    "Busy is a poor synonym for useful.",
    "Schedule thinking time like a meeting.",
    "Be the calmest person in the room.",
    "Make your future easy; leave breadcrumbs.",
    "If it isn't documented, it didn't happen.",
    "Good names save hours.",
    "Saying no is how you say yes to what matters.",
    "Don't multiply entities beyond necessity.",
    "Batteries die; plans shouldn't.",
    "The simplest solution is often invisible behind ego.",
    "Checklists beat memory every day of the week.",
    "If you're bored, you're not asking better questions.",
    "Consistency outruns intensity.",
    "Move in pencil until you must ink.",
    "A deadline is a design constraint.",
    "If you can't measure it, you're guessing.",
    "Trade clever for clear.",
    "Premature optimization is the tax on fear.",
    "Write code your future self won't subtweet.",
    "Version control is a time machine. Name your commits.",
    "A little friction prevents big fires.",
    "Learn names. Remember birthdays. Ship value.",
    "You rise to your systems, not your goals.",
    "Debugging is diplomacy with computers.",
    "Strong opinions, loosely held, kindly expressed.",
    "Be the person who writes the migration guide.",
    "The right abstraction removes a thousand lines.",
    "Focus is saying no to good ideas.",
    "Work expands to fill the vagueness available.",
    "If you can't delete it, you don't own it.",
    "Shallow work screams; deep work whispers.",
    "A good sign saves a thousand questions.",
    "Patience loves company: breathe wisely.",
    "Edge moments are where surprises live.",
    "Notes: write for humans under stress.",
    "Availability is a chain; find the weakest link.",
    "Your promises are contracts with tomorrow.",
    "No is a complete sentence.",
    "Default to public praise and private critique.",
    "Tools rust without practice.",
    "Curate inputs like you curate friendships.",
    "Ask 'what problem dies if we stop doing this?'.",
    "Nobody regrets a polite follow‑up.",
    "If it's important, automate the reminder.",
    "The more you care, the clearer you should write.",
    "Design for the next maintainer.",
    "The user is trying to get back to their life.",
    "Great teams argue about ideas, not people.",
    "Meeting ended on time? That's a feature.",
    "Be curious longer than you're defensive.",
    "A little gratitude changes the room temperature.",
    "Estimate in ranges; commit to checkpoints.",
    "When in doubt, sketch.",
    "Use fewer words and stronger ones.",
    "Make it obvious. Then make it elegant.",
    "Good defaults beat good docs.",
    "Write the error that would have saved you yesterday.",
    "Be allergic to brittle.",
    "Clean up before you clock out.",
    "Teams that demo, deliver.",
    "What you repeat defines culture.",
    "Less grind, more glide.",
    "Don't break a sweat to save a second.",
    "Prefer boring technology that works.",
    "Empathy is the root dependency.",
    "Make rollback easy and rare.",
    "The map is not the mesh.",
    "Constraints are creativity in disguise.",
    "Seek signal; mute noise.",
    "Write once for humans, twice for machines.",
    "Small PRs, big smiles.",
    "Scope is the valve on stress.",
    "Ship with pride, not fear.",
    "Leave things better than you found them.",
    "Kind is a career strategy.",
    "Today's tiny improvement is tomorrow's standard.",
    "If it took heroics, it wasn't a process.",
    "Prefer clarity over certainty.",
    "Test your assumptions, not your patience.",
    "Meetings are expensive; spend wisely.",
    "If nobody owns it, the user does.",
    "Protect focus like it's billable.",
    "The right constraint unlocks ideas.",
    "Hold the line on scope creep.",
    "Your schedule is a budget for attention.",
    "A well-named function is a love letter to readers.",
    "Deep work requires closed tabs.",
    "What gets celebrated gets repeated.",
    "Simplicity ages well.",
    "A good comment explains the why, not the what.",
    "Make onboarding a product.",
    "If it's flaky, it's failing.",
    "Choose explicit over implicit.",
    "Pick your future regrets.",
    "If it hurts, do it more often until it doesn't.",
    "Practice is how talent pays rent.",
    "Documentation is hospitality in text.",
    "Stop asking for time; ask for tradeoffs.",
    "A budget is a list of pre-approved regrets.",
    "Work that matters rarely feels urgent.",
    "Don't outsource your judgment to dashboards.",
    "Bad news early is good news.",
    "Courage is a habit of small brave acts.",
    "You don't need more time; you need fewer tasks.",
    "Calm is contagious.",
    "Your environment is a silent teammate.",
    "Routines free creativity to roam.",
    "Never make big promises right before you leave.",
    "Fix the rough edges your users trip on.",
    "Prefer progress you can prove.",
    "Make success easy to spot.",
    "Find the constraint, feed the flow.",
    "The opposite of focus is drift.",
    "You don't rise to goals; you fall to systems.",
    "First impressions linger longer than you think.",
    "A single clear owner beats a committee.",
    "Ask what a beginner would misunderstand.",
    "Performance is a feature for everyone.",
    "Stop starting; start finishing.",
    "If a shortcut needs a map, it's not shorter.",
    "Optimize for boring reliability.",
    "Every list is a map of your hopes.",
    "The best metric is the one you act on.",
    "Design like you're explaining to your past self.",
    "If the root cause starts with 'someone should', try again.",
    "Soft skills are hard and compound.",
    "Ask for context, not just answers.",
    "Write down the decision while it's fresh.",
    "Your future self reads your diary entries.",
    "A good test is a story with receipts.",
    "Automation is a mirror; polish before you reflect.",
    "Don't be clever; be kind to the reader.",
    "Schedule breaks before burnout schedules you.",
    "Default to written notes over fuzzy memories.",
    "The next error message should teach.",
    "If it's too fragile to refactor, it's telling you something.",
    "Narrow the scope until you can move.",
    "Find leverage, not just effort.",
    "Do the thing you are avoiding for five minutes.",
    "Every 'quick fix' leaves a footprint.",
    "Delete is a powerful feature; use carefully.",
    "Great UX feels like telepathy.",
    "Delays you can't see still cost smiles.",
    "Don't stack meetings back to back with thinking.",
    "Move the labels with the boxes.",
    "A demo beats a thousand words.",
    "Write the playbook you wish you had.",
    "An experiment without a stop date is scope creep.",
    "Practice gratitude out loud.",
    "Ship small, learn fast, sleep well.",
    "Design for failure paths first.",
    "Leave a map when you wander off-road.",
    "A setback is a class; take good notes.",
    "Names are invitations to understanding.",
    "Your first draft isn't supposed to be good; it's supposed to exist.",
    "Celebrate deletion: less to maintain.",
    "If you dread it, pair it.",
    "A crisp constraint is a gift.",
    "If the answer isn't obvious, try a tiny experiment.",
    "Timebox exploration; schedule exploitation.",
    "Work visible, worries smaller.",
    "A checklist turns chaos into choreography.",
    "If everything looks perfect, check your assumptions.",
    "Prefer clear words to clever ones.",
    "If it's hard to test, it's hard to trust.",
    "Honor the off switch.",
    "Balance is a verb.",
    "Refuse to be the single point of failure.",
    "Problems like dark corners; add light.",
    "Don't turn urgency into an identity.",
    "A spec is a promise; keep it short and clear.",
    "Delete stale tabs and stale beliefs.",
    "Reduce cognitive load like it's latency.",
    "Ask 'what would great look like in two sentences?'.",
    "Take notes as if someone else will need them.",
    "Speak last if you hold power.",
    "Kind feedback starts with context, not conclusions.",
    "Assume good intent; verify with evidence.",
    "Choose boring defaults and exciting outcomes.",
    "A small buffer prevents a large panic.",
    "Trust, but verify kindly.",
    "Batch decisions; stream kindness.",
    "The best time to simplify was yesterday; the second best is now.",
    "Practice saying 'I don't know' followed by 'let's find out'.",
    "Guardrails beat guard dogs.",
    "Use manners like seatbelts: always on.",
    "Emergencies should be rare and never routine.",
    "If you need a hero, fix the system.",
    "An apology is not a strategy.",
    "Act as if your mentor is watching.",
    "The line between brittle and elegant is testing.",
    "Start with the smallest honest promise.",
    "Prefer fewer obligations, more presence.",
    "Make the happy path likely, and the failure path kind.",
    "If you must be fast, first be calm.",
    "Friction is feedback. Listen before sanding.",
    "Work with time, not against it.",
    "Distractions are toll booths on your attention.",
    "The calendar is a crowd of intentions. Be selective.",
    "Curate your mornings like front door locks.",
    "Sane limits are kind defaults.",
    "Ship a story, not a shrug.",
    "A good question can replace a bad meeting.",
    "Move decisions to where information lives.",
    "Respect the reader's time like it's your own.",
    "Align incentives; everything else is friction.",
    "Better is fragile; protect it with process.",
    "Use checklists to catch sleepy moments.",
    "The unglamorous path is often the fastest.",
    "Work on the thing only you can do.",
    "Make it hard to do the wrong thing.",
    "Opinionated tools reduce decision fatigue.",
    "A clear exit is the start of a good plan.",
    "Make limits visible to make tradeoffs honest.",
    "Every tradition was once a new idea that stuck.",
    "Be a thermostat, not a thermometer.",
    "Time you enjoy wasting isn't wasted; schedule it.",
    "Ask for the smallest useful version.",
    "Errors should feel like potholes, not cliffs.",
    "Keep wisdom in notebooks, not just in heads.",
    "If it's manual, it's a future outage.",
    "Prefer clear agreements to vague nods.",
    "Put your ego on mute; listen generously.",
    "Generosity turns smart into wise.",
    "A good routine beats a thousand hacks.",
    "Be precise with problems and generous with people.",
    "The best time to write docs was when it made sense; the second best is now.",
    "Separate what is true from what you wish were true.",
    "Make the invisible visible: queues, limits, costs.",
    "Ship learning, not just code.",
    "Be the teammate who brings clarity.",
    "A promise without a date is a wish.",
    "Stability is a feature users can feel.",
    "A small kindness travels far.",
    "If the stakes are high, rehearse.",
    "Make your defaults reversible.",
    "Write less, say more.",
    "Limit WIP: Work In Progress is work in peril.",
    "Every constraint is a question in disguise.",
    "Protect mornings for thinking.",
    "Reduce handoffs; increase ownership.",
    "Remove the sharp edges in your product and your process.",
    "Hold your tools lightly; hold your principles tight.",
    "Expect surprises; design for forgiveness.",
    "If it's confusing now, it's chaos later.",
    "Treat legacy with respect; it kept the lights on.",
    "Be ambitious about quality, realistic about time.",
    "Trust is uptime for humans.",
    "A kind review invites a better revision.",
    "If you need a hero every Friday, fire your calendar.",
    "Make escalation rare by making ownership clear.",
    "Celebrate teams, not saviors.",
    "The best documentation starts a week before you need it.",
    "A system reveals its values in its defaults.",
    "Mentoring is the fastest way to learn twice.",
    "Ask 'what would we remove if we had to cut 30%?'.",
    "Speed limits belong in systems and schedules.",
    "Use warmth to make truth easier to hear.",
    "If your metrics need a tour guide, simplify them.",
    "Design like a host; greet the user and guide them home.",
    "Every 'later' must have a when.",
    "Work with reality, not your wish list.",
    "A crisp 'no' is kinder than a vague 'maybe'.",
    "Be strict with systems and gentle with people.",
    "If a policy needs an exception every week, fix the policy.",
    "Practice graceful exits in plans and conversations.",
    "Give credit like confetti.",
    "Repeat the mission until it's muscle memory.",
    "Tidiness is an act of courtesy.",
    "Prefer portable habits to flashy shortcuts.",
    "A little prep makes luck look like skill.",
    "Observe before you optimize.",
    "If the checklist is long, make two.",
    "Turn risks into rehearsals.",
    "Tell the honest story you would sign your name to.",
    "Let data inform, not dictate.",
    "A single sentence can align a week of work.",
    "Reduce options to reduce regret.",
    "Grace under pressure is a practiced skill.",
    "If it's important, say it twice—once in speech, once in writing.",
    "Make delight the default path.",
    "Your first draft is for you; your second is for them.",
    "Work visible, gratitude audible.",
    "Clear beats clever every time.",
    "Set expectations, then exceed them gently.",
    "Make the right behavior the path of least resistance.",
    "A crisis is just a poorly scheduled rehearsal.",
    "If it's hard to name, it's hard to defend or improve.",
    "Don't let perfect bully good.",
    "Be generous with credit and specific with thanks.",
    "Constraints protect craft.",
    "Design is a promise; keep it small and true.",
    "Your backlog is not a wish list; it's a queue.",
    "Teach your habits your taste.",
    "Kind feedback teaches taste.",
    "Reduce surprises with smaller steps.",
    "Make the invisible handrails obvious.",
    "A retry without a reason is a loop.",
    "The best reminder is the one that saved a day.",
    "Practice like a dress rehearsal, not a storage unit.",
    "Prefer edits to opinions.",
    "Every status update should say what's next.",
    "Make good habits cheap and bad habits expensive.",
    "Ask less 'who' and more 'how did this happen?'.",
    "Your habits are the story your days tell you.",
    "If you can't name it, you can't change it.",
    "Ruthlessly protect quiet time.",
    "Plan backwards from the demo.",
    "Delete one sentence from every email.",
    "An easy off‑switch is a kindness.",
    "Stability is strategy you can sleep with.",
    "Assume nothing; pay attention to everything.",
    "The best price for complexity is not paying it.",
    "Polish the moments people feel the most.",
    "If trust is low, communication must be high.",
    "A little whitespace is mercy for minds.",
    "Practice the pause before replies.",
    "A good apology is specific, sincere, and soon.",
    "Plan for rain; celebrate sun.",
    "Write tests like seatbelts: you never regret them.",
    "Teach your future self with today's commit.",
    "Don't let the urgent steal from the important.",
    "Prototype the riskiest assumption first.",
    "Retrospectives are how teams get younger.",
    "Design for power outages and people outages.",
    "A graceful error beats a silent success.",
    "Make opting out easy and respectful.",
    "Most meetings could be a memo; most memos could be a sentence.",
    "Ship value, not just versions.",
    "Hire for kindness; train for craft.",
    "Be the teammate you needed last week.",
    "Tend your tools like a garden.",
    "Small acts of clarity prevent big acts of heroism.",
    "A well-placed tooltip is a tiny tutorial.",
    "Respect weekends; protect Wednesdays.",
    "If it's not checked, it's probably not done.",
    "Reduce choices, increase chances.",
    "The right word at the right time is magic.",
    "Own the outcome, share the spotlight.",
    "Take your time; make it count.",
    "A gentle nudge beats a loud reminder.",
    "Leave space in your schedule for serendipity.",
    "If it scales your kindness, automate it.",
    "Be the calm in someone else's sprint.",
    "Clarity first, speed second.",
    "Choose defaults you would recommend to a friend.",
    "Make the handoff a handshake, not a toss.",
    "A single example is better than a page of adjectives.",
    "If you need less confusion, add more labels.",
    "Track decisions as if you’ll need to defend them.",
    "Start the day by finishing something small.",
    "Teach your calendar how to say no.",
    "Good taste is curated constraints.",
    "Be present in the meeting you're in, not the one after.",
    "Polish is how you show respect.",
    "Edit your estimates with experience.",
    "When the stakes rise, slow the pace.",
    "Make the default secure and the secure simple.",
    "Let the user feel clever for doing the right thing.",
    "A good tool changes how you think.",
    "Make do‑overs safe and simple.",
    "Your character is what you do on a bad day.",
    "Give people small ways to win often.",
    "Be generous with context, stingy with complexity.",
    "A little humor buys a lot of patience.",
    "Teach with examples; persuade with stories.",
    "Make the first minute delightful.",
    "If you must choose, choose humane.",
    "A kind 'no' today is a better 'yes' later.",
    "Don't outrun your headlights.",
    "Guard your attention like a scarce resource.",
    "If the fix needs a novel, fix the system.",
    "Celebrate boring outages that never happen.",
    "Your future thanks you for deleting code.",
    "Make maintenance a first-class feature.",
    "If it's confusing in the code, it’s confusing in the UI.",
    "Ship like you mean it, rest like you need it.",
    "Trust is earned by small, consistent truths.",
    "Change one thing at a time when the stakes are high.",
    "If you can't test it, you can't keep it.",
    "Teach your product to say 'sorry' well.",
    "A little margin prevents a lot of mess.",
    "The only unreadable code is tomorrow's.",
    "Plan your week like a favorite trip.",
    "The right ritual makes good work inevitable.",
    "Let your work speak, then help it be heard.",
    "Be the reason the playbook gets better.",
];

/// Pick a random fortune from the classic database.
/// 
/// Returns a reference to a static string containing a fortune cookie message.
/// All fortunes are guaranteed to be under 200 characters for mesh network compatibility.
/// 
/// # Examples
/// 
/// ```
/// use meshbbs::bbs::fortune::get_fortune;
/// 
/// let fortune = get_fortune();
/// assert!(!fortune.is_empty());
/// assert!(fortune.len() <= 200);
/// ```
/// 
/// # Thread Safety
/// 
/// This function is thread-safe and uses `rand::thread_rng()` for randomization.
/// Multiple calls from different threads will produce independent random results.
pub fn get_fortune() -> &'static str {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..FORTUNES.len());
    FORTUNES[idx]
}

/// Get the total number of fortunes in the database.
/// 
/// This function is primarily useful for testing and diagnostics.
/// 
/// # Examples
/// 
/// ```
/// use meshbbs::bbs::fortune::fortune_count;
/// 
/// assert_eq!(fortune_count(), 573);
/// ```
pub fn fortune_count() -> usize {
    FORTUNES.len()
}

/// Get the maximum length of any fortune in the database.
/// 
/// This function is useful for validation and ensuring all fortunes
/// meet the mesh network size constraints.
/// 
/// # Examples
/// 
/// ```
/// use meshbbs::bbs::fortune::max_fortune_length;
/// 
/// assert!(max_fortune_length() <= 200);
/// ```
pub fn max_fortune_length() -> usize {
    FORTUNES.iter().map(|f| f.len()).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fortunes_count_573() {
        assert_eq!(FORTUNES.len(), 573);
    }

    #[test]
    fn all_fortunes_under_200_chars() {
        for (i, fortune) in FORTUNES.iter().enumerate() {
            assert!(
                fortune.len() <= 200,
                "Fortune {} is too long ({} chars): {}",
                i, fortune.len(), fortune
            );
        }
    }

    #[test]
    fn all_fortunes_non_empty() {
        for (i, fortune) in FORTUNES.iter().enumerate() {
            assert!(
                !fortune.is_empty(),
                "Fortune {} is empty",
                i
            );
        }
    }

    #[test]
    fn all_fortunes_contain_printable_chars() {
        for (i, fortune) in FORTUNES.iter().enumerate() {
            assert!(
                fortune.chars().all(|c| !c.is_control() || c.is_ascii_whitespace()),
                "Fortune {} contains control characters: {}",
                i, fortune
            );
        }
    }

    #[test]
    fn fortune_returns_valid_response() {
        let fortune = get_fortune();
        assert!(!fortune.is_empty());
        assert!(fortune.len() <= 200);
        assert!(FORTUNES.contains(&fortune));
    }

    #[test]
    fn fortune_randomness_check() {
        // Run multiple times to ensure we get different results
        let mut results = std::collections::HashSet::new();
        for _ in 0..50 {
            results.insert(get_fortune());
        }
        // Should get at least 10 different fortunes in 50 tries
        assert!(results.len() >= 10, "Fortune randomness seems poor: only {} unique results", results.len());
    }

    #[test]
    fn fortune_thread_safety_simulation() {
        // Simulate concurrent access by calling get_fortune many times rapidly
        let mut handles = vec![];
        
        for _ in 0..10 {
            let handle = std::thread::spawn(|| {
                let mut local_results = std::collections::HashSet::new();
                for _ in 0..20 {
                    local_results.insert(get_fortune());
                }
                local_results
            });
            handles.push(handle);
        }
        
        let mut all_results = std::collections::HashSet::new();
        for handle in handles {
            let thread_results = handle.join().unwrap();
            all_results.extend(thread_results);
        }
        
        // Should collect a good variety of fortunes across threads
        assert!(all_results.len() >= 15, "Concurrent access produced only {} unique fortunes", all_results.len());
    }

    #[test]
    fn fortune_database_quality_checks() {
        // Check that we have a good mix of different types of content
        let programming_related = FORTUNES.iter().filter(|f| {
            f.to_lowercase().contains("code") || 
            f.to_lowercase().contains("program") ||
            f.to_lowercase().contains("computer") ||
            f.to_lowercase().contains("software")
        }).count();
        
        let philosophical = FORTUNES.iter().filter(|f| {
            f.contains("Socrates") || 
            f.contains("Aristotle") ||
            f.contains("Einstein") ||
            f.contains("wisdom")
        }).count();
        
        // Ensure we have a reasonable distribution
        assert!(programming_related >= 10, "Should have at least 10 programming-related fortunes, found {}", programming_related);
        assert!(philosophical >= 5, "Should have at least 5 philosophical fortunes, found {}", philosophical);
    }

    #[test]
    fn fortune_count_matches_array() {
        assert_eq!(fortune_count(), FORTUNES.len());
        assert_eq!(fortune_count(), 573);
    }

    #[test]
    fn max_fortune_length_validation() {
        let max_len = max_fortune_length();
        assert!(max_len <= 200, "Maximum fortune length {} exceeds 200 character limit", max_len);
        assert!(max_len > 0, "Maximum fortune length should be greater than 0");
        
        // Verify it actually matches the longest fortune
        let actual_max = FORTUNES.iter().map(|f| f.len()).max().unwrap();
        assert_eq!(max_len, actual_max);
    }

    #[test]
    fn helper_functions_consistency() {
        // Ensure helper functions are consistent with the actual data
        assert_eq!(fortune_count(), FORTUNES.len());
        
        let calculated_max = FORTUNES.iter().map(|f| f.len()).max().unwrap_or(0);
        assert_eq!(max_fortune_length(), calculated_max);
    }
}