from fontTools.ttLib import TTFont

font = TTFont('icons7.otf')
name_table = font['name']

# Print all name records
for record in name_table.names:
    print(f"ID {record.nameID}: {record.toUnicode()}")

# Get specific names
print(f"Font Family: {name_table.getBestFamilyName()}")
print(f"Subfamily: {name_table.getBestSubFamilyName()}")
print(f"Full Name: {name_table.getBestFullName()}")