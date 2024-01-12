#!/usr/bin/env python
import json

out = []

# You can get the default-cards from bulk data from scryfall
# timestamp will be different
with open("default-cards-20240111220517.json") as f:
    j = json.load(f)

    for card in j:
        type_line = card.get("type_line")

        if card["legalities"]["commander"] != "legal":
            continue

        if not "paper" in card["games"]:
            continue

        # Skip stuff like Brisela
        if card["layout"] == "meld":
            continue

        if card.get("card_faces") != None and card["card_faces"][0]["type_line"]:
            type_line = card["card_faces"][0]["type_line"]


        if type_line:
            if "Creature" in type_line and "Legendary" in type_line:
                out.append(card["name"])
            elif "Background" in type_line:
                out.append(card["name"])
            elif card.get("oracle_text") != None and "can be your commander" in card["oracle_text"]:
                out.append(card["name"])

out = list(set(out))
out.sort()

with open("out.json", 'w') as f:
    json.dump(out, f)

