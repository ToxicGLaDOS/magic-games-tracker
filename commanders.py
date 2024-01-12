#!/usr/bin/env python
import json

out = []

# You can get the default-cards from bulk data from scryfall
# timestamp will be different
with open("default-cards-20240111220517.json") as f:
    j = json.load(f)

    for card in j:
        type_line = card.get("type_line")
        name = card.get("name")

        if card["legalities"]["commander"] != "legal":
            continue

        if not "paper" in card["games"]:
            continue

        # Skip stuff like Brisela
        all_parts = card.get("all_parts")
        if all_parts:
            meld_result_parts = list(filter(lambda part: part["component"] == "meld_result", all_parts))
            if len(meld_result_parts) > 0:
                meld_result_part = meld_result_parts[0]
                if meld_result_part["name"] == name:
                    continue

        if card.get("card_faces") != None and card["card_faces"][0]["type_line"]:
            type_line = card["card_faces"][0]["type_line"]
            name = card["card_faces"][0]["name"]

        if type_line:
            if "Creature" in type_line and "Legendary" in type_line:
                out.append(name)
            elif "Background" in type_line:
                out.append(name)
            elif card.get("oracle_text") != None and "can be your commander" in card["oracle_text"]:
                out.append(name)
            elif name == "Grist, the Hunger Tide":
                out.append(name)

out = list(set(out))
out.sort()

with open("out.json", 'w') as f:
    json.dump(out, f, indent=4)

