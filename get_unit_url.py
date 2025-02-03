# Script for systematically getting all the unit portraits for all feh units in the spreadsheet

SOURCE_URL: str = "https://fireemblemwiki.org/wiki/Category:Heroes_portraits"
UNIT_GALLERY_CLASS_NAME: str = "gallery"
URL_HEADER_INDEX_END: int = 51
IM_URL_FILE_NAME: str = "./data/FEH_Unit_URLs.txt"
SPACE = " "
NEW_LINE = "\n"

from selenium import webdriver
from selenium.webdriver.common.by import By
from bs4 import BeautifulSoup

import time
import io
import itertools

# Parse the name of a unit
def get_unit_name(unit_descript: str) -> str:
    unit_name: str = ""
    for character in itertools.takewhile(lambda c : c != "_", unit_descript):
        unit_name += character
    return unit_name

# Parse the title of a unit
def get_title(unit_descript: str) -> str:
    unit_name: str = ""
    for character in itertools.takewhile(lambda c : c != ".", unit_descript):
        unit_name += character
    return unit_name[0:-4]

# Write FEH Unit Image URLs to file
def extract_page_info(driver: webdriver.Edge, fd: io.TextIOWrapper) -> None:
    for unit_tag in BeautifulSoup(driver.page_source, "html.parser").find_all(name="ul", class_=UNIT_GALLERY_CLASS_NAME)[0].find_all("img"):
        unit_link: str = unit_tag["src"]

        # STRING PARSING -> getting the full url, the unit name, and the unit title
        full_unit_descriptor: str = unit_link[URL_HEADER_INDEX_END:len(unit_link)]
        unit_name: str = get_unit_name(full_unit_descriptor)
        unit_title: str = get_title(full_unit_descriptor[(len(unit_name) + 1):len(full_unit_descriptor)])

        fd.write(unit_name + SPACE + unit_title + SPACE + unit_link + NEW_LINE)

# Navigates to the next page. If the operation was successful we return true,
# otherwise false is returned.
def next_page(driver: webdriver.Edge) -> bool:
    success: bool = True
    try:
        elem = driver.find_element(By.LINK_TEXT, "next page")
        webdriver.ActionChains(driver).click(elem).perform()
    except:
        success = False
    return success

def main() -> int:
    driver: webdriver.Edge = webdriver.Edge()
    driver.get(SOURCE_URL)

    with open(IM_URL_FILE_NAME, "a") as fd:
        has_next: bool = True
        while has_next:
            time.sleep(3.0)
            extract_page_info(driver, fd)
            has_next = next_page(driver)
    
    driver.close()
    return 0

if __name__ == "__main__":
    exit_code: int = main()
    print(f"Returned with exit code {exit_code}")